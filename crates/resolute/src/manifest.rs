use std::{
	collections::HashMap,
	path::PathBuf,
	time::{Duration, SystemTime},
};

use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::{
	fs,
	io::{self, AsyncReadExt},
};
use url::Url;

use crate::{Error, Result};

/// Default Resonite mod manifest JSON file URL (from the Resonite Modding Group repository)
pub const MANIFEST_URL: &str =
	"https://raw.githubusercontent.com/resonite-modding-group/resonite-mod-manifest/main/manifest.json";

/// Downloads the manifest JSON and caches it if necessary
pub async fn download(config: &ManifestConfig) -> Result<String> {
	// Download the manifest
	info!("Downloading manifest from {}", config.remote_url);
	let response = reqwest::get(config.remote_url.clone()).await?;
	let status = response.status();
	if !status.is_success() {
		return Err(Error::Http(status));
	}
	let json = response.text().await?;

	// Cache the JSON to disk if necessary
	if let Some(cache) = &config.cache_file_path {
		info!("Caching manifest to {}", cache.display());
		let mut file = fs::OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(cache)
			.await?;
		io::copy(&mut json.as_bytes(), &mut file).await?;
	}

	Ok(json)
}

/// Obtains the manifest JSON either from the cache (if it exists and isn't stale) or by downloading it
pub async fn retrieve(config: &ManifestConfig) -> Result<String> {
	// If we don't have a cache file path, go ahead and do a download
	let Some(cache) = &config.cache_file_path else {
		return download(config).await;
	};

	match fs::OpenOptions::new().read(true).open(cache).await {
		Ok(mut file) => {
			// Ensure the cache isn't stale - if it is, we try downloading the manifest instead.
			// If that fails for any reason, we'll use the cache anyway.
			if let Some(stale_after) = config.cache_stale_after {
				let modified = file.metadata().await?.modified()?;
				let stale_time = modified
					.checked_add(stale_after)
					.expect("unable to calculate cache staleness threshold");
				if SystemTime::now().gt(&stale_time) {
					info!(
						"Manifest cache is stale (older than {} seconds) - redownloading",
						stale_after.as_secs()
					);
					match download(config).await {
						Ok(json) => return Ok(json),
						Err(err) => warn!("Failed to download manifest to replace stale cache: {}", err),
					}
				}
			}

			// Read the JSON from the cache
			info!("Reading from manifest cache at {}", cache.display());
			let mut json = String::new();
			file.read_to_string(&mut json).await?;
			Ok(json)
		}

		Err(err) => {
			warn!("Error opening manifest cache: {}", err);
			download(config).await
		}
	}
}

/// Deserializes manifest JSON
pub fn parse(json: &str) -> Result<ManifestData> {
	Ok(serde_json::from_str(json)?)
}

/// Configuration for the manifest
#[derive(Clone, Debug)]
pub struct ManifestConfig {
	pub remote_url: Url,
	pub cache_file_path: Option<PathBuf>,
	pub cache_stale_after: Option<Duration>,
}

impl ManifestConfig {
	/// Creates a new manifest configuration with defaults set
	pub fn new() -> Self {
		Self {
			remote_url: Url::parse(MANIFEST_URL).expect("cannot parse default manifest url"),
			cache_file_path: None,
			cache_stale_after: Some(Duration::from_secs(60 * 60 * 6)),
		}
	}

	/// Sets the URL of the remote manifest file for downloads
	pub fn url<U>(mut self, url: U) -> Self
	where
		U: TryInto<Url>,
		<U as TryInto<Url>>::Error: std::fmt::Debug,
	{
		self.remote_url = url.try_into().expect("unable to parse given url");
		self
	}

	/// Sets the cache file path to use
	pub fn cache(mut self, path: PathBuf) -> Self {
		self.cache_file_path = Some(path);
		self
	}

	/// Disables caching (and clears any cache file path that was previously set)
	pub fn no_cache(mut self) -> Self {
		self.cache_file_path = None;
		self
	}

	/// Marks the cache as stale after a provided duration
	pub fn stale_after(mut self, duration: Duration) -> Self {
		self.cache_stale_after = Some(duration);
		self
	}

	/// Ensures the cache is never considered stale
	pub fn never_stale(mut self) -> Self {
		self.cache_stale_after = None;
		self
	}
}

impl Default for ManifestConfig {
	fn default() -> Self {
		Self::new()
	}
}

/// Represents the top-level object in the manifest JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestData {
	pub objects: ManifestObjects,
	#[serde(rename = "schemaVersion")]
	pub schema_version: String,
}

/// Represents the "objects" object in the manifest JSON
pub type ManifestObjects = HashMap<String, ManifestObject>;

/// Represents a single "objects" entry in the manifest JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestObject {
	#[serde(rename = "author")]
	pub authors: ManifestAuthors,
	pub entries: ManifestEntries,
}

/// Represents a "author" object in the manifest JSON
pub type ManifestAuthors = HashMap<String, ManifestAuthor>;

/// Represents a single "author" entry in the manifest JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestAuthor {
	pub url: Option<Url>,
	pub icon: Option<Url>,
	pub support: Option<Url>,
}

/// Represents an "entries" object in the manifest JSON
pub type ManifestEntries = HashMap<String, ManifestEntry>;

/// Represents a single "entries" entry in the manifest JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestEntry {
	pub name: String,
	pub description: String,
	pub category: String,
	#[serde(rename = "sourceLocation")]
	pub source_location: Option<Url>,
	pub website: Option<Url>,
	pub tags: Option<Vec<String>>,
	pub flags: Option<Vec<String>>,
	pub platforms: Option<Vec<String>>,
	#[serde(rename = "additionalAuthors")]
	pub additional_authors: Option<ManifestAuthors>,
	pub versions: ManifestEntryVersions,
}

/// Represents a "versions" object in the manifest JSON
pub type ManifestEntryVersions = HashMap<String, ManifestEntryVersion>;

/// Represents a single "versions" entry in the manifest JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestEntryVersion {
	pub artifacts: Vec<ManifestEntryArtifact>,
	pub dependencies: Option<ManifestEntryDependencies>,
	pub conflicts: Option<ManifestEntryDependencies>,
	#[serde(rename = "releaseUrl")]
	pub release_url: Option<Url>,
}

/// Represents a single "artifacts" entry in the manifest JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestEntryArtifact {
	pub url: Url,
	pub sha256: String,
	pub filename: Option<String>,
	#[serde(rename = "installLocation")]
	pub install_location: Option<String>,
}

/// Represents a "dependencies" or "conflicts" object in the manifest JSON
pub type ManifestEntryDependencies = HashMap<String, ManifestEntryDependency>;

/// Represents a single "dependencies" or "conflicts" entry in the manifest JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestEntryDependency {
	pub version: String,
}
