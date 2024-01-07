mod delete;
mod download;
mod paths;

use std::path::{Path, PathBuf};

use log::debug;
use semver::Version;

#[cfg(feature = "db")]
use crate::db::ResoluteDatabase;
use crate::mods::{self, ResoluteMod, ResoluteModMap};
use crate::{discover, manifest, Error, Result};

pub use self::delete::Deleter;
pub use self::download::Downloader;
pub use self::download::DownloaderBuilder;
pub use self::paths::ArtifactPaths;

/// Main entry point for all mod-related operations that need to be persisted
pub struct ModManager<#[cfg(feature = "db")] 'a> {
	#[cfg(feature = "db")]
	pub db: ResoluteDatabase<'a>,
	pub downloader: Downloader,
	pub deleter: Deleter,
	base_dest: PathBuf,
	http_client: reqwest::Client,
}

macro_rules! impl_ModManager_with_without_db {
	{ impl ModManager $implementations:tt } => {
		#[cfg(feature = "db")]
		impl<'a> ModManager<'a> $implementations

		#[cfg(not(feature = "db"))]
		impl ModManager $implementations
	}
}

impl_ModManager_with_without_db! {
	impl ModManager {
		/// Creates a new mod manager
		pub fn new(
			#[cfg(feature = "db")] db: ResoluteDatabase<'a>,
			base_dest: impl AsRef<Path>,
			http_client: &reqwest::Client,
		) -> Self {
			Self {
				#[cfg(feature = "db")]
				db,
				downloader: DownloaderBuilder::new()
					.base(&base_dest)
					.http_client(http_client.clone())
					.build(),
				deleter: Deleter::new(&base_dest),
				http_client: http_client.clone(),
				base_dest: base_dest.as_ref().to_path_buf(),
			}
		}

		/// Gets all mods that have a version installed
		#[cfg(feature = "db")]
		pub async fn get_installed_mods(&self) -> Result<ResoluteModMap> {
			let mods = tokio::task::block_in_place(move || -> Result<ResoluteModMap> {
				Ok(self
					.db
					.get_installed_mods()?
					.into_iter()
					.map(|rmod| (rmod.id.clone(), rmod))
					.collect())
			})?;
			Ok(mods)
		}

		/// Gets all mods from a manifest, and if the "db" feature is active, marks any installed ones
		pub async fn get_all_mods(&self, manifest_config: manifest::Config, bypass_cache: bool) -> Result<ResoluteModMap> {
			let manifest = manifest::Client::new(manifest_config, self.http_client.clone());

			// Retrieve the manifest JSON
			let json = if !bypass_cache {
				manifest.retrieve().await
			} else {
				debug!("Forcing download of manifest");
				manifest.download().await
			}?;

			// Parse the JSON into raw manifest data, load that into a mod map
			let mut mods = tokio::task::spawn_blocking(move || -> Result<ResoluteModMap> {
				let data = manifest.parse(&json)?;
				let mods = mods::load_manifest(data);
				Ok(mods)
			})
			.await??;

			#[cfg(feature = "db")]
			self.mark_installed_mods(&mut mods).await?;

			Ok(mods)
		}

		/// Fills in the installed_version field for all mods in a map that are installed and
		/// adds any necessary missing versions to the mods' versions maps
		#[cfg(feature = "db")]
		pub async fn mark_installed_mods(&self, mods: &mut ResoluteModMap) -> Result<()> {
			let installed_mods = self.get_installed_mods().await?;

			for (id, rmod) in mods.iter_mut() {
				if let Some(installed) = installed_mods.get(id) {
					// Set the installed version from the stored mod
					let semver = installed.installed_version.clone();
					rmod.installed_version = semver.clone();

					// Add the version to the mod's version map if it doesn't have it
					if let Some(semver) = semver {
						if !rmod.versions.contains_key(&semver) {
							rmod.versions.insert(
								semver.clone(),
								installed
									.versions
									.get(&semver)
									.ok_or_else(|| Error::UnknownVersion(installed.name.clone(), semver))?
									.clone(),
							);
						}
					}
				}
			}

			Ok(())
		}

		/// Installs a mod, and if the "db" feature is active, stores it as installed in the database
		pub async fn install_mod<P>(&self, rmod: &ResoluteMod, version: impl AsRef<str>, progress: P) -> Result<()>
		where
			P: Fn(u64, u64),
		{
			// Determine the version to install
			let semver = Version::parse(version.as_ref())?;
			let version = rmod
				.versions
				.get(&semver)
				.ok_or_else(|| Error::UnknownVersion(rmod.id.clone(), semver))?;

			// Download the version and add the mod to the database
			self.downloader.download_version(version, progress).await?;
			#[cfg(feature = "db")]
			{
				let mut rmod = rmod.clone();
				rmod.installed_version = Some(version.semver.clone());
				tokio::task::block_in_place(|| self.db.store_mod(rmod))?;
			}

			Ok(())
		}

		/// Installs a new version of a mod and removes any remaining artifacts from the previous version
		pub async fn update_mod<P>(&self, rmod: &ResoluteMod, version: impl AsRef<str>, progress: P) -> Result<()>
		where
			P: Fn(u64, u64),
		{
			// Ensure the mod is actually installed and determine which version
			let old_version = {
				let version = match &rmod.installed_version {
					Some(version) => version,
					None => return Err(Error::ModNotInstalled(Box::new(rmod.clone()))),
				};
				rmod.versions
					.get(version)
					.ok_or_else(|| Error::UnknownVersion(rmod.id.clone(), version.to_owned()))?
			};

			// Determine the new version to install
			let semver = Version::parse(version.as_ref())?;
			let new_version = rmod
				.versions
				.get(&semver)
				.ok_or_else(|| Error::UnknownVersion(rmod.id.clone(), semver))?;

			// Install the new version and remove any left over artifacts
			self.install_mod(rmod, new_version.semver.to_string(), progress).await?;
			self.deleter
				.delete_artifacts_diff(&old_version.artifacts, &new_version.artifacts)
				.await?;

			Ok(())
		}

		/// Uninstalls a mod's installed version
		pub async fn uninstall_mod(&self, rmod: &ResoluteMod) -> Result<()> {
			// Ensure the mod is actually installed and determine which version
			let installed_version = match &rmod.installed_version {
				Some(version) => rmod
					.versions
					.get(version)
					.ok_or_else(|| Error::UnknownVersion(rmod.id.clone(), version.clone()))?,
				None => return Err(Error::ModNotInstalled(Box::new(rmod.clone()))),
			};

			// Delete the version artifacts and remove the mod from the database
			self.deleter.delete_version(installed_version).await?;
			#[cfg(feature = "db")]
			tokio::task::block_in_place(|| self.db.remove_mod(&rmod.id))?;

			Ok(())
		}

		/// Discovers any installed mods in the base path, and if the "db" feature is active, stores them in the database
		pub async fn discover_installed_mods(&self, manifest_config: manifest::Config) -> Result<ResoluteModMap> {
			let all_mods = self.get_all_mods(manifest_config, false).await?;

			let resonite_path = self.base_dest.clone();
			let discovered = tokio::task::block_in_place(|| discover::discover_mods(resonite_path, all_mods))?;

			#[cfg(feature = "db")]
			tokio::task::block_in_place(|| {
				for rmod in discovered.values() {
					debug!("Storing discovered mod {}", rmod);
					self.db.store_mod(rmod.clone())?;
				}
				Ok::<_, Error>(())
			})?;

			Ok(discovered)
		}

		/// Changes the base destination of mods for the manager
		pub fn set_base_dest(&mut self, path: impl AsRef<Path>) {
			let path = path.as_ref();
			self.base_dest = path.to_owned();
			self.downloader.base_dest = path.to_owned();
			self.deleter.base_dest = path.to_owned();
		}
	}
}
