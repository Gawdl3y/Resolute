use std::path::Path;

use futures_util::TryStreamExt;
use path_clean::PathClean;
use reqwest::{Client, IntoUrl};
use tokio::{
	fs,
	io::{AsyncWriteExt, BufWriter},
};

use crate::mods::{ModArtifact, ModVersion};
use crate::{Error, Result};

/// Handles mod downloads
pub struct Downloader {
	client: Client,
}

impl Downloader {
	pub fn new(client: Client) -> Self {
		Self { client }
	}

	/// Downloads all relevant artifacts for a specific mod version to their proper destinations in the given base path
	pub async fn download_version<P>(&self, version: &ModVersion, base_dest: &Path, progress: P) -> Result<()>
	where
		P: Fn(u64, u64),
	{
		for artifact in version.artifacts.iter() {
			self.download_artifact(artifact, base_dest, &progress).await?;
		}

		Ok(())
	}

	/// Downloads a specific artifact to its proper destination in the given base path
	pub async fn download_artifact<P>(&self, artifact: &ModArtifact, base_dest: &Path, progress: P) -> Result<()>
	where
		P: Fn(u64, u64),
	{
		// Add the artifact's install location to the path
		let mut dest = base_dest.join(match &artifact.install_location {
			Some(install_location) => {
				let path = Path::new(install_location);
				path.strip_prefix("/").or::<Error>(Ok(path))?
			}
			None => Path::new("rml_mods"),
		});

		// Add the artifact's filename to the path
		match &artifact.filename {
			Some(filename) => dest.push(filename),
			None => dest.push(
				Path::new(artifact.url.path())
					.file_name()
					.ok_or(Error::Path("unable to extract file name from url".to_owned()))?,
			),
		};

		// Ensure the final path is inside the base path
		let final_dest = dest.clean();
		if !final_dest.starts_with(base_dest) {
			return Err(Error::Path(
				"artifact's final destination is not a subdirectory of the base destination".to_owned(),
			));
		}

		// Create any missing directories up to the destination
		let result = fs::create_dir_all(final_dest.parent().ok_or(Error::Path(
			"unable to get parent of artifact's final destination".to_owned(),
		))?)
		.await;

		// If the directory creation failed, ignore the error it if it's just because it already exists
		if let Err(err) = result {
			if err.kind() == std::io::ErrorKind::AlreadyExists {
				return Err(Error::Io(err));
			}
		}

		self.download(artifact.url.clone(), dest.as_path(), progress).await
	}

	/// Downloads a file to a destination path, emitting progress updates along the way
	pub(crate) async fn download<P>(&self, url: impl IntoUrl, dest: &Path, progress: P) -> Result<()>
	where
		P: Fn(u64, u64),
	{
		// Make the request
		let request = self.client.get(url);
		let response = request.send().await?;

		// Ensure the request yielded a successful response
		let status = response.status();
		if !status.is_success() {
			return Err(Error::Http(status));
		}

		// Prep the file and stream
		let total_bytes = response.content_length().unwrap_or(0);
		let mut file = BufWriter::new(fs::File::create(dest).await?);
		let mut stream = response.bytes_stream();

		// Write each chunk to the file and send a progress update
		while let Some(chunk) = stream.try_next().await? {
			file.write_all(&chunk).await?;
			progress(chunk.len() as u64, total_bytes);
		}

		file.flush().await?;
		Ok(())
	}
}

impl Default for Downloader {
	fn default() -> Self {
		Self::new(reqwest::Client::new())
	}
}
