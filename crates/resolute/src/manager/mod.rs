pub(crate) mod artifacts;
mod delete;
mod download;

use std::path::{Path, PathBuf};

use log::debug;
use semver::Version;
use serde::{Deserialize, Serialize};
use tokio::task;

#[cfg(feature = "db")]
use crate::db::ResoluteDatabase;
use crate::models::{self, ResoluteMod, ResoluteModMap};
use crate::{discover, manifest, Error, Result};

pub use self::delete::Deleter;
pub use self::download::Downloader;
pub use self::download::DownloaderBuilder;

/// Main entry point for all mod-related operations that need to be persisted
#[allow(missing_debug_implementations, clippy::module_name_repetitions)]
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
			http_client: reqwest::Client,
		) -> Self {
			Self {
				#[cfg(feature = "db")]
				db,
				downloader: DownloaderBuilder::new()
					.base(&base_dest)
					.http_client(http_client.clone())
					.build(),
				deleter: Deleter::new(&base_dest),
				http_client,
				base_dest: base_dest.as_ref().to_path_buf(),
			}
		}

		/// Gets all mods that have a version installed
		#[cfg(feature = "db")]
		#[allow(clippy::unused_async)]
		pub async fn get_installed_mods(&self) -> Result<LoadedMods> {
			let mods = task::block_in_place(move || -> Result<LoadedMods> {
				let mods = self
					.db
					.get_installed_mods()?
					.into_iter()
					.map(|rmod| (rmod.id.clone(), rmod))
					.collect();

				Ok(LoadedMods { mods, removed: None })
			})?;
			Ok(mods)
		}

		/// Gets all mods from a manifest, and if the "db" feature is active, marks any installed ones.
		/// Returns a tuple of the mods and any removed mods (if applicable).
		pub async fn get_all_mods(&self, manifest_config: manifest::Config, bypass_cache: bool) -> Result<LoadedMods> {
			let manifest = manifest::Client::new(manifest_config, self.http_client.clone());

			// Retrieve the manifest JSON
			let json = if bypass_cache {
				debug!("Forcing download of manifest");
				manifest.download().await
			} else {
				manifest.retrieve().await
			}?;

			// Parse the JSON into raw manifest data, load that into a mod map
			let mut mods = task::spawn_blocking(move || -> Result<ResoluteModMap> {
				let data = manifest.parse(&json)?;
				let mods = models::load_manifest(data);
				Ok(mods)
			})
			.await??;

			#[cfg(feature = "db")]
			let removed = self.mark_installed_mods(&mut mods).await?;
			#[cfg(not(feature = "db"))]
			let removed = None;

			Ok(LoadedMods { mods, removed })
		}

		/// Fills in the installed_version field for all mods in a map that are installed and
		/// adds any necessary missing versions to the mods' versions maps.
		/// Returns a list of unrecognized mods that should be considered removed, if any.
		#[cfg(feature = "db")]
		pub async fn mark_installed_mods(&self, mods: &mut ResoluteModMap) -> Result<Option<ResoluteModMap>> {
			use std::{collections::HashSet, ffi::OsString};

			use crate::models::{ModArtifact, ModVersion};

			// Load the installed mods
			let LoadedMods {
				mods: installed_mods, ..
			} = self.get_installed_mods().await?;

			// Get all unrecognized mods
			let unrecognized_mods: Vec<&ResoluteMod> =
				installed_mods.values().filter(|rmod| rmod.is_unrecognized()).collect();

			let mut removed_mods = ResoluteModMap::new();

			for (id, rmod) in mods.iter_mut() {
				if let Some(installed) = installed_mods.get(id) {
					// Set the installed version from the stored mod
					let semver = installed.installed_version.clone();
					rmod.installed_version.clone_from(&semver);

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

				if !unrecognized_mods.is_empty() {
					// Get the latest version of the known mod
					let Some(latest_version) = rmod.latest_version() else {
						continue;
					};

					// Build a set of artifact filenames for the known mod
					let expected_artifact_names: HashSet<OsString> = latest_version
						.artifacts
						.iter()
						.map(ModArtifact::usable_filename)
						.collect();

					// Find unrecognized mods that contain any of the filenames
					let unrecognized_matches: Vec<&ResoluteMod> = unrecognized_mods
						.iter()
						.filter(|umod| {
							// Get the latest version of the unrecognized mod
							let Some(latest_version) = umod.latest_version() else {
								return false;
							};

							// Build a set of artifact filenames for the unrecognized mod
							let artifact_names: HashSet<OsString> = latest_version
								.artifacts
								.iter()
								.map(ModArtifact::usable_filename)
								.collect();

							// Include this unrecognized mod only if all of its artifacts are included in the known mod's
							artifact_names.difference(&expected_artifact_names).count() == 0
						})
						.copied()
						.collect();

					if unrecognized_matches.is_empty() {
						continue;
					}

					// Create a new unrecognized version for the mod with the combined artifacts from the unrecognized mods
					let unrecognized_artifacts: Vec<ModArtifact> = unrecognized_matches
						.iter()
						.flat_map(|umod| {
							let latest_version = umod.latest_version().expect("no latest version for unrecognized mod");
							latest_version.artifacts.iter().cloned()
						})
						.collect();
					let unrecognized_version = ModVersion::new_unrecognized_with_artifacts(unrecognized_artifacts);

					// Add the version to the mod's versions map and mark it as the installed version
					let semver = unrecognized_version.semver.clone();
					rmod.versions.insert(semver.clone(), unrecognized_version);
					rmod.installed_version = Some(semver);

					// Replace the unrecognized mods with the known mod in the database
					task::block_in_place(|| -> Result<()> {
						removed_mods.reserve(unrecognized_matches.len());

						for umod in unrecognized_matches {
							debug!(
								"Removing unrecognized mod {} from the database during installed mod marking, repacing with {}",
								umod,
								rmod,
							);
							self.db.remove_mod_by_id(&umod.id)?;
							removed_mods.insert(umod.id.clone(), umod.clone());
						}

						self.db.store_mod(rmod.clone())?;
						Ok(())
					})?;
				}
			}

			Ok(if removed_mods.is_empty() {
				None
			} else {
				Some(removed_mods)
			})
		}

		/// Installs a mod, and if the "db" feature is active, stores it as installed in the database
		pub async fn install_mod(
			&self,
			rmod: &ResoluteMod,
			version: impl AsRef<str>,
			progress: impl Fn(u64, u64),
		) -> Result<()> {
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
				task::block_in_place(|| self.db.store_mod(rmod))?;
			}

			Ok(())
		}

		/// Installs a new version of a mod and removes any remaining artifacts from the previous version
		pub async fn update_mod(
			&self,
			rmod: &ResoluteMod,
			version: impl AsRef<str>,
			progress: impl Fn(u64, u64),
		) -> Result<()> {
			// Ensure the mod is actually installed and determine which version
			let old_version = {
				let Some(version) = &rmod.installed_version else {
					return Err(Error::ModNotInstalled(Box::new(rmod.clone())));
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
			task::block_in_place(|| self.db.remove_mod_by_id(&rmod.id))?;

			Ok(())
		}

		/// Discovers any installed mods in the base path, and if the "db" feature is active, stores them in the database
		pub async fn discover_installed_mods(&self, manifest_config: manifest::Config) -> Result<ResoluteModMap> {
			let LoadedMods { mods: all_mods, .. } = self.get_all_mods(manifest_config, false).await?;

			let resonite_path = self.base_dest.clone();
			let discovered = task::block_in_place(|| discover::mods(resonite_path, &all_mods))?;

			#[cfg(feature = "db")]
			task::block_in_place(|| {
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
			path.clone_into(&mut self.base_dest);
			path.clone_into(&mut self.downloader.base_dest);
			path.clone_into(&mut self.deleter.base_dest);
		}

		/// Changes the HTTP client to use for downloads
		pub fn set_http_client(&mut self, http_client: reqwest::Client) {
			self.downloader.http_client = http_client.clone();
			self.http_client = http_client;
		}
	}
}

/// A set of loaded mods from a manager
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::exhaustive_structs)]
pub struct LoadedMods {
	pub mods: ResoluteModMap,
	pub removed: Option<ResoluteModMap>,
}
