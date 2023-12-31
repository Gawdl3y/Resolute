use std::path::{Path, PathBuf};

use futures_util::TryStreamExt;
use log::{debug, error, info};
use reqwest::{Client, IntoUrl};
use sha2::{Digest, Sha256};
use tokio::{
	fs,
	io::{AsyncWriteExt, BufWriter},
};

use crate::mods::{ModArtifact, ModVersion};
use crate::{Error, Result};

use super::ArtifactPaths;

/// Handles mod downloads
pub struct Downloader {
	pub base_dest: PathBuf,
	pub http_client: Client,
}

impl Downloader {
	pub fn new(base_dest: impl AsRef<Path>, http_client: Client) -> Self {
		Self {
			http_client,
			base_dest: base_dest.as_ref().to_owned(),
		}
	}

	/// Downloads all relevant artifacts for a specific mod version to their proper destinations in the given base path
	pub async fn download_version<P>(&self, version: &ModVersion, progress: P) -> Result<()>
	where
		P: Fn(u64, u64),
	{
		let mut install_error = None;

		// Download all of the artifacts and track any successful ones - on an error, abort any further ones
		let mut downloaded = Vec::new();
		for artifact in version.artifacts.iter() {
			match self.download_artifact(artifact, &progress).await {
				Ok(paths) => downloaded.push(paths),
				Err(err) => {
					install_error = Some(err);
					break;
				}
			}
		}

		// If an error was encountered, delete the downloaded ones
		if let Some(err) = install_error {
			error!("Error downloading artifacts, undoing any successful ones: {}", err);

			for paths in downloaded {
				let _ = self.cancel_artifact(&paths).await;
			}

			return Err(err);
		}

		// Finalize the artifacts and track any successful ones - on an error, abort any further ones
		let mut finalized = Vec::new();
		for paths in downloaded {
			match self.finalize_artifact(&paths).await {
				Ok(_) => finalized.push(paths),
				Err(err) => {
					install_error = Some(err);
					break;
				}
			}
		}

		// If an error was encountered, delete the finalized ones and return the old artifacts to their original locations
		if let Some(err) = install_error {
			error!(
				"Error finalizing downloaded artifacts, undoing any successful ones: {}",
				err
			);

			for paths in finalized {
				let _ = self.delete_artifact(&paths).await;
				let _ = self.return_old_artifact(&paths).await;
			}

			return Err(err);
		}

		// Delete the old artifacts
		let _ = self.delete_old_artifacts(&finalized).await;
		Ok(())
	}

	/// Downloads a specific artifact to a temporary destination (filename.dll.new) within a given base path
	pub async fn download_artifact<P>(&self, artifact: &ModArtifact, progress: P) -> Result<ArtifactPaths>
	where
		P: Fn(u64, u64),
	{
		let paths = ArtifactPaths::try_new(artifact, &self.base_dest)?;

		// Create any missing directories up to the destination
		let result = fs::create_dir_all(
			paths
				.final_dest
				.parent()
				.ok_or_else(|| Error::Path("unable to get parent of artifact's final destination".to_owned()))?,
		)
		.await;

		// If the directory creation failed, ignore the error it if it's just because it already exists
		if let Err(err) = result {
			if err.kind() != std::io::ErrorKind::AlreadyExists {
				return Err(Error::Io(err));
			}
		}

		// Download the artifact to its temporary location
		info!("Downloading artifact {} to {}", artifact.url, paths.tmp_dest.display());
		self.download(
			artifact.url.clone(),
			paths.tmp_dest.as_path(),
			&artifact.sha256,
			progress,
		)
		.await?;

		Ok(paths)
	}

	/// Moves a downloaded artifact from its temporary destination to its final one
	pub async fn finalize_artifact(&self, paths: &ArtifactPaths) -> Result<()> {
		// Try renaming any old file that may exist and ignore the error if it doesn't
		let result = fs::rename(&paths.final_dest, &paths.old_dest).await;
		if let Err(err) = result {
			if err.kind() != std::io::ErrorKind::NotFound {
				return Err(Error::Io(err));
			}
		} else {
			debug!(
				"Renamed old artifact file {} to {}",
				paths.final_dest.display(),
				paths.old_dest.display()
			);
		}

		// Rename the downloaded file from its temporary name to its final one
		fs::rename(&paths.tmp_dest, &paths.final_dest).await?;
		debug!(
			"Renamed temporary artifact file {} to {}",
			paths.tmp_dest.display(),
			paths.final_dest.display()
		);

		Ok(())
	}

	/// Deletes a downloaded artifact from its temporary destination
	pub async fn cancel_artifact(&self, paths: &ArtifactPaths) -> Result<()> {
		fs::remove_file(&paths.tmp_dest).await?;
		debug!("Deleted temporary artifact file {}", paths.tmp_dest.display());
		Ok(())
	}

	/// Returns an old artifact that was moved to a temporary location to its original location
	pub async fn return_old_artifact(&self, paths: &ArtifactPaths) -> Result<()> {
		fs::rename(&paths.old_dest, &paths.final_dest).await?;
		debug!(
			"Renamed old artifact file {} to {}",
			paths.old_dest.display(),
			paths.final_dest.display()
		);

		Ok(())
	}

	/// Deletes a downloaded artifact from its final destination
	pub async fn delete_artifact(&self, paths: &ArtifactPaths) -> Result<()> {
		fs::remove_file(&paths.final_dest).await?;
		debug!("Deleted artifact file {}", paths.final_dest.display());
		Ok(())
	}

	/// Deletes the old artifacts that were moved to a temporary location
	pub async fn delete_old_artifacts(&self, artifact_paths: &Vec<ArtifactPaths>) -> Result<()> {
		for paths in artifact_paths {
			fs::remove_file(&paths.old_dest).await?;
			debug!("Deleted old artifact file {}", paths.old_dest.display());
		}

		Ok(())
	}

	/// Downloads a file to a destination path and checks its integrity, emitting progress updates along the way
	pub(crate) async fn download<P>(
		&self,
		url: impl IntoUrl + Into<String> + Clone,
		dest: impl AsRef<Path>,
		checksum: &str,
		progress: P,
	) -> Result<()>
	where
		P: Fn(u64, u64),
	{
		let dest = dest.as_ref();

		// Make the request
		let request = self.http_client.get(url.clone());
		let response = request.send().await?;

		// Ensure the request yielded a successful response
		let status = response.status();
		if !status.is_success() {
			return Err(Error::Http(status));
		}

		// Prep the file, stream, and hasher
		let total_bytes = response.content_length().unwrap_or(0);
		let mut file = BufWriter::new(fs::File::create(dest).await?);
		let mut stream = response.bytes_stream();
		let mut hasher = Sha256::new();

		// Write each chunk to the file and send a progress update
		while let Some(chunk) = stream.try_next().await? {
			file.write_all(&chunk).await?;
			hasher.update(&chunk);
			progress(chunk.len() as u64, total_bytes);
		}
		file.flush().await?;

		// Verify the integrity of the downloaded file - if it doesn't match, delete the file
		let digest = hasher.finalize();
		let actual = format!("{:x}", digest);
		if actual != checksum.to_lowercase() {
			let _ = fs::remove_file(dest).await;
			return Err(Error::Checksum {
				expected: checksum.to_owned(),
				checksum: actual,
				file: url.into(),
			});
		}

		debug!("Downloaded artifact to {}", dest.display());
		Ok(())
	}
}

#[derive(Default)]
pub struct DownloaderBuilder {
	base_dest: PathBuf,
	http_client: Client,
}

impl DownloaderBuilder {
	/// Creates a new builder with defaults set
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the base destination of mod artifacts
	pub fn base(mut self, base_dest: impl AsRef<Path>) -> Self {
		self.base_dest = base_dest.as_ref().to_owned();
		self
	}

	/// Sets the HTTP client to use
	pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
		self.http_client = http_client;
		self
	}

	/// Creates a Client using this builder's configuration and HTTP client
	pub fn build(self) -> Downloader {
		Downloader::new(self.base_dest, self.http_client)
	}
}
