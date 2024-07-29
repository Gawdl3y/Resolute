use std::{
	fmt,
	io::ErrorKind,
	path::{Path, PathBuf},
};

use futures_util::TryStreamExt;
use log::{debug, error, info};
use reqwest::{Client, IntoUrl};
use sha2::{Digest, Sha256};
use tokio::{
	fs,
	io::{AsyncWriteExt, BufWriter},
};

use crate::models::{ModArtifact, ModVersion};
use crate::{Error, Result};

use super::artifacts::{self, ArtifactAction, ArtifactError, MappableToArtifactError};

/// Handles mod downloads
#[derive(Debug)]
#[non_exhaustive]
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
	pub async fn download_version(&self, version: &ModVersion, progress: impl Fn(u64, u64)) -> Result<()> {
		let mut install_error = None;

		// Download all of the artifacts and track any successful ones - on an error, abort any further ones
		let mut downloaded = Vec::new();
		for artifact in &version.artifacts {
			match self.download_artifact(artifact, &progress).await {
				Ok(dl_artifact) => downloaded.push(dl_artifact),
				Err(err) => {
					install_error = Some(err);
					break;
				}
			}
		}

		// If an error was encountered, delete the downloaded ones
		if let Some(err) = install_error {
			error!("Error downloading artifacts, canceling any successful ones: {}", err);

			for dl_artifact in downloaded {
				let artifact = dl_artifact.artifact;
				if let Err(err) = dl_artifact.cancel().await {
					error!("Error canceling downloaded artifact ({}.new): {}", artifact, err);
				}
			}

			return Err(err);
		}

		// Finalize the artifacts and track any successful ones - on an error, abort any further ones
		let mut finalized = Vec::new();
		for dl_artifact in downloaded {
			match dl_artifact.finalize().await {
				Ok(final_artifact) => finalized.push(final_artifact),
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

			for final_artifact in finalized {
				let artifact = final_artifact.artifact;
				if let Err(err) = final_artifact.undo().await {
					error!("Error undoing actions for finalized artifact ({}): {}", artifact, err);
				}
			}

			return Err(err);
		}

		// Delete the old artifacts
		for final_artifact in finalized.into_iter().filter(FinalizedArtifact::has_old) {
			let artifact = final_artifact.artifact;
			if let Err(err) = final_artifact.delete_old().await {
				error!("Error deleting old artifact ({}.old): {}", artifact, err);
			}
		}

		Ok(())
	}

	/// Downloads a specific artifact to a temporary destination (filename.dll.new) within a given base path
	pub async fn download_artifact<'a>(
		&self,
		artifact: &'a ModArtifact,
		progress: impl Fn(u64, u64),
	) -> Result<DownloadedArtifact<'a>> {
		let final_dest = artifact
			.dest_within(&self.base_dest)
			.map_pathless_artifact_err(ArtifactAction::Download)?;
		let tmp_dest = ModArtifact::tmp_dest(&final_dest).map_pathless_artifact_err(ArtifactAction::Download)?;

		// Create any missing directories up to the destination
		let result = fs::create_dir_all(tmp_dest.parent().ok_or_else(|| ArtifactError {
			action: ArtifactAction::Download,
			path: None,
			source: Box::new(Error::Path(
				"unable to get parent of artifact's temporary destination".to_owned(),
			)),
		})?)
		.await;

		// If the directory creation failed, ignore the error it if it's just because it already exists
		if let Err(err) = result {
			if err.kind() != ErrorKind::AlreadyExists {
				return Err(Error::Artifact(ArtifactError {
					action: ArtifactAction::Download,
					path: Some(tmp_dest),
					source: Box::new(err.into()),
				}));
			}
		}

		// Download the artifact to its temporary location
		info!("Downloading artifact {} to {}", artifact.url, tmp_dest.display());
		self.download(artifact.url.clone(), &tmp_dest, &artifact.sha256, progress)
			.await
			.map_artifact_err(ArtifactAction::Download, &tmp_dest)?;

		Ok(DownloadedArtifact {
			artifact,
			final_dest,
			tmp_dest,
		})
	}

	/// Downloads a file to a destination path and checks its integrity, emitting progress updates along the way
	pub(crate) async fn download(
		&self,
		url: impl IntoUrl + Into<String> + Clone,
		dest: impl AsRef<Path>,
		checksum: &str,
		progress: impl Fn(u64, u64),
	) -> Result<()> {
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
		let actual = format!("{digest:x}");
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

/// Builder for a [Downloader]
#[derive(Default, Debug)]
pub struct DownloaderBuilder {
	base_dest: PathBuf,
	http_client: Client,
}

impl DownloaderBuilder {
	/// Creates a new builder with defaults set
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the base destination of mod artifacts
	#[must_use]
	pub fn base(mut self, base_dest: impl AsRef<Path>) -> Self {
		base_dest.as_ref().clone_into(&mut self.base_dest);
		self
	}

	/// Sets the HTTP client to use
	#[must_use]
	pub fn http_client(mut self, http_client: Client) -> Self {
		self.http_client = http_client;
		self
	}

	/// Creates a Client using this builder's configuration and HTTP client
	#[must_use]
	pub fn build(self) -> Downloader {
		Downloader::new(self.base_dest, self.http_client)
	}
}

/// An artifact that has been downloaded by a [Downloader]
#[derive(Debug)]
#[must_use]
pub struct DownloadedArtifact<'a> {
	pub artifact: &'a ModArtifact,
	final_dest: PathBuf,
	tmp_dest: PathBuf,
}

impl<'a> DownloadedArtifact<'a> {
	/// Finalizes the artifact.
	/// If an artifact already exists at the final destination, it gets renamed with a temporary suffix.
	/// The downloaded artifact at the temporary destination is then renamed to its final destination.
	pub async fn finalize(self) -> Result<FinalizedArtifact<'a>> {
		let old_dest = ModArtifact::old_dest(&self.final_dest).map_pathless_artifact_err(ArtifactAction::Rename)?;

		// Rename the old file if one exists
		let has_old = artifacts::rename(&self.final_dest, &old_dest, true).await?;
		if has_old {
			debug!(
				"Renamed old artifact file {} to {}",
				self.final_dest.display(),
				old_dest.display()
			);
		}

		// Rename the downloaded file from its temporary name to its final one
		artifacts::rename(&self.tmp_dest, &self.final_dest, false).await?;
		debug!(
			"Renamed temporary artifact file {} to {}",
			self.tmp_dest.display(),
			self.final_dest.display()
		);

		Ok(FinalizedArtifact {
			artifact: self.artifact,
			final_dest: self.final_dest,
			old_dest: has_old.then_some(old_dest),
		})
	}

	/// Cancels the artifact. Deletes the downloaded file from the temporary destination.
	pub async fn cancel(self) -> Result<()> {
		artifacts::delete(&self.tmp_dest, true).await?;
		debug!("Deleted temporary artifact file {}", self.tmp_dest.display());
		Ok(())
	}
}

impl fmt::Display for DownloadedArtifact<'_> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.artifact.fmt(f)
	}
}

/// An artifact that has been downloaded and finalized
#[derive(Debug)]
#[must_use]
pub struct FinalizedArtifact<'a> {
	pub artifact: &'a ModArtifact,
	final_dest: PathBuf,
	old_dest: Option<PathBuf>,
}

impl FinalizedArtifact<'_> {
	/// Completely undoes any actions performed for the artifact.
	/// Deletes the downloaded artifact and renames the old artifact back to its original final destination.
	pub async fn undo(self) -> Result<()> {
		// Delete the artifact from its final destination
		artifacts::delete(&self.final_dest, true).await?;
		debug!("Deleted artifact file {}", self.final_dest.display());

		// Rename the old artifact back to the final name if there was one
		if let Some(old_dest) = self.old_dest {
			artifacts::rename(&old_dest, &self.final_dest, false).await?;
			debug!(
				"Renamed old artifact file {} to {}",
				old_dest.display(),
				self.final_dest.display()
			);
		}

		Ok(())
	}

	/// Deletes the old artifact file, if there was one.
	/// Fails if there was no old artifact or if there is an issue during its deletion.
	pub async fn delete_old(self) -> Result<()> {
		let old_dest = self.old_dest.ok_or_else(|| Error::NoOldArtifact)?;
		artifacts::delete(&old_dest, true).await?;
		debug!("Deleted old artifact file {}", old_dest.display());
		Ok(())
	}

	/// Checks whether there is an old artifact file present that was renamed
	#[must_use]
	pub const fn has_old(&self) -> bool {
		self.old_dest.is_some()
	}
}

impl fmt::Display for FinalizedArtifact<'_> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.artifact.fmt(f)
	}
}
