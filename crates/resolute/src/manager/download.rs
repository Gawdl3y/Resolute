use std::{
	fmt::Display,
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

use crate::mods::{ModArtifact, ModVersion};
use crate::{Error, Result};

/// Handles mod downloads
#[derive(Debug)]
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
		for final_artifact in finalized.into_iter().filter(|artifact| artifact.has_old()) {
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
		let final_dest = artifact.dest_within(&self.base_dest)?;

		// Create any missing directories up to the destination
		let result = fs::create_dir_all(
			final_dest
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
		let tmp_dest = ModArtifact::tmp_dest(&final_dest)?;
		info!("Downloading artifact {} to {}", artifact.url, tmp_dest.display());
		self.download(artifact.url.clone(), &tmp_dest, &artifact.sha256, progress)
			.await?;

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

/// Builder for a [Downloader]
#[derive(Default, Debug)]
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
		let mut has_old = false;

		// Try renaming any old file that may exist and ignore the error if it doesn't
		let old_dest = ModArtifact::old_dest(&self.final_dest)?;
		if let Err(err) = fs::rename(&self.final_dest, &old_dest).await {
			if err.kind() != std::io::ErrorKind::NotFound {
				return Err(Error::Io(err));
			}
		} else {
			has_old = true;
			debug!(
				"Renamed old artifact file {} to {}",
				self.final_dest.display(),
				old_dest.display()
			);
		}

		// Rename the downloaded file from its temporary name to its final one
		fs::rename(&self.tmp_dest, &self.final_dest).await?;
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
		fs::remove_file(&self.tmp_dest).await?;
		debug!("Deleted temporary artifact file {}", self.tmp_dest.display());
		Ok(())
	}
}

impl Display for DownloadedArtifact<'_> {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
		fs::remove_file(&self.final_dest).await?;
		debug!("Deleted artifact file {}", self.final_dest.display());

		// Rename the old artifact back to the final name if there was one
		if let Some(old_dest) = self.old_dest {
			fs::rename(&old_dest, &self.final_dest).await?;
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
		fs::remove_file(&old_dest).await?;
		debug!("Deleted old artifact file {}", old_dest.display());
		Ok(())
	}

	/// Checks whether there is an old artifact file present that was renamed
	pub fn has_old(&self) -> bool {
		self.old_dest.is_some()
	}
}

impl Display for FinalizedArtifact<'_> {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.artifact.fmt(f)
	}
}
