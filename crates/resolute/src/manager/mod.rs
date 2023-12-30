mod download;
mod paths;

use std::path::Path;

use log::info;

#[cfg(feature = "db")]
use crate::db::ResoluteDatabase;
use crate::mods::{self, ResoluteMod, ResoluteModMap};
use crate::{manifest, Error, Result};

pub use self::download::Downloader;
pub use self::download::DownloaderBuilder;
pub use self::paths::ArtifactPaths;

/// Main entry point for all mod-related operations that need to be persisted
pub struct ModManager<#[cfg(feature = "db")] 'a> {
	#[cfg(feature = "db")]
	pub db: ResoluteDatabase<'a>,
	pub downloader: Downloader,
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
				http_client: http_client.clone(),
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
				info!("Forcing download of manifest");
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

		/// Fills in the installed_version field for all mods in a map that are installed
		#[cfg(feature = "db")]
		pub async fn mark_installed_mods(&self, mods: &mut ResoluteModMap) -> Result<()> {
			let installed_mods = self.get_installed_mods().await?;

			for (id, rmod) in mods.iter_mut() {
				if let Some(installed) = installed_mods.get(id) {
					rmod.installed_version = installed.installed_version.clone();
				}
			}

			Ok(())
		}

		/// Installs a mod, and if the "db" feature is active, stores it as installed in the database
		pub async fn install_mod<P>(&self, rmod: &ResoluteMod, version: impl AsRef<str>, progress: P) -> Result<()>
		where
			P: Fn(u64, u64),
		{
			let version = rmod
				.versions
				.get(version.as_ref())
				.ok_or_else(|| Error::UnknownVersion(rmod.id.clone(), version.as_ref().to_owned()))?;

			self.downloader.download_version(version, progress).await?;

			#[cfg(feature = "db")]
			{
				let mut rmod = rmod.clone();
				rmod.installed_version = Some(version.semver.clone());
				self.db.store_mod(rmod)?;
			}

			Ok(())
		}

		/// Changes the base destination of mods for the manager
		pub fn set_base_dest(&mut self, path: impl AsRef<Path>) {
			let path = path.as_ref();
			self.downloader.base_dest = path.to_owned();
		}
	}
}
