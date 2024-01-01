use std::collections::HashSet;
use std::path::{Path, PathBuf};

use log::info;
use tokio::fs;

use crate::error::{ArtifactAction, ArtifactError, ArtifactErrorVec};
use crate::mods::{ModArtifact, ModVersion};
use crate::{Error, Result};

use super::ArtifactPaths;

/// Handles deleting mods
pub struct Deleter {
	pub base_dest: PathBuf,
}

impl Deleter {
	pub fn new(base_dest: impl AsRef<Path>) -> Self {
		Self {
			base_dest: base_dest.as_ref().to_owned(),
		}
	}

	/// Deletes all installed artifacts for a specific mod version
	pub async fn delete_version(&self, version: &ModVersion) -> Result<()> {
		// Delete all artifacts and track any failed ones
		let mut failed = ArtifactErrorVec::new();
		for artifact in version.artifacts.iter() {
			self.delete_artifact(artifact)
				.await
				.map(|_| ())
				.or_else(|(err, paths)| {
					failed.push(ArtifactError {
						action: ArtifactAction::Delete,
						path: paths.map(|paths| paths.final_dest),
						source: Box::new(err),
					});
					Ok::<_, Error>(())
				})?;
		}

		if failed.is_empty() {
			Ok(())
		} else {
			Err(Error::Artifacts(failed))
		}
	}

	/// Deletes a single artifact
	pub async fn delete_artifact(
		&self,
		artifact: &ModArtifact,
	) -> core::result::Result<ArtifactPaths, (Error, Option<ArtifactPaths>)> {
		let paths = ArtifactPaths::try_new(artifact, &self.base_dest).map_err(|err| (err, None))?;
		fs::remove_file(&paths.final_dest)
			.await
			.map_err(|err| (err.into(), Some(paths.clone())))?;

		info!("Deleted artifact file {}", paths.final_dest.display());
		Ok(paths)
	}

	/// Deletes leftover artifacts from an old set that aren't present in a new set of artifacts
	pub async fn delete_artifacts_diff(
		&self,
		old_artifacts: &[ModArtifact],
		new_artifacts: &[ModArtifact],
	) -> Result<()> {
		// Build a list of paths for both the new and old artifacts
		let new_paths = new_artifacts
			.iter()
			.map(|artifact| ArtifactPaths::try_new(artifact, &self.base_dest))
			.collect::<Result<HashSet<ArtifactPaths>>>()?;
		let old_paths: HashSet<ArtifactPaths> = old_artifacts
			.iter()
			.map(|artifact| ArtifactPaths::try_new(artifact, &self.base_dest))
			.collect::<Result<HashSet<ArtifactPaths>>>()?;

		// Determine the paths that are no longer needed
		let unnecessary_paths: Vec<&ArtifactPaths> = old_paths.difference(&new_paths).collect();

		// Delete each unnecessary artifact path and track any failures
		let mut failed = ArtifactErrorVec::new();
		for paths in unnecessary_paths {
			fs::remove_file(&paths.final_dest).await.map(|_| ()).or_else(|err| {
				failed.push(ArtifactError {
					action: ArtifactAction::Delete,
					path: Some(paths.final_dest.clone()),
					source: Box::new(Error::Io(err)),
				});
				Ok::<_, Error>(())
			})?;
			info!("Deleted left-over artifact file {}", paths.final_dest.display());
		}

		if failed.is_empty() {
			Ok(())
		} else {
			Err(Error::Artifacts(failed))
		}
	}
}
