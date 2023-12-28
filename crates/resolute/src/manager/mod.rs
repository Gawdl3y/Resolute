mod download;
mod paths;

use std::path::Path;

use reqwest::Client;

#[cfg(feature = "db")]
use crate::db::ResoluteDatabase;
use crate::mods::ResoluteMod;
use crate::{Error, Result};

pub use self::download::Downloader;
pub use self::download::DownloaderBuilder;
pub use self::paths::ArtifactPaths;

/// Main entry point for all mod-related operations that need to be persisted
pub struct ModManager<#[cfg(feature = "db")] 'a> {
	#[cfg(feature = "db")]
	pub db: ResoluteDatabase<'a>,
	pub downloader: Downloader,
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
			http_client: &Client,
		) -> Self {
			Self {
				#[cfg(feature = "db")] db,
				downloader: DownloaderBuilder::new()
					.base(&base_dest)
					.http_client(http_client.clone())
					.build(),
			}
		}

		/// Installs a mod and stores it as installed in the database
		pub async fn install_mod<P>(&self, rmod: &ResoluteMod, version: impl AsRef<str>, progress: P) -> Result<()>
		where
			P: Fn(u64, u64),
		{
			let version = rmod
				.versions
				.get(version.as_ref())
				.ok_or_else(|| Error::UnknownVersion(rmod.id.clone(), version.as_ref().to_owned()))?;

			self.downloader.download_version(version, progress).await?;

			#[cfg(feature = "db")] {
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
