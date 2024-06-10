use std::collections::HashSet;
use std::path::{Path, PathBuf};

use log::info;

use crate::mods::{ModArtifact, ModVersion};
use crate::Error;

use super::artifacts::{self, ArtifactAction, ArtifactError, ArtifactErrorVec, MappableToArtifactError};

/// Handles deleting mods
#[derive(Debug)]
#[non_exhaustive]
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
	pub async fn delete_version(&self, version: &ModVersion) -> Result<(), ArtifactErrorVec> {
		// Delete all artifacts and track any failed ones
		let mut failed = ArtifactErrorVec::new();
		for artifact in &version.artifacts {
			if let Err(err) = self.delete_artifact(artifact).await {
				failed.push(err);
			}
		}

		if failed.is_empty() {
			Ok(())
		} else {
			Err(failed)
		}
	}

	/// Deletes a single artifact
	pub async fn delete_artifact(&self, artifact: &ModArtifact) -> Result<PathBuf, ArtifactError> {
		let path = artifact
			.dest_within(&self.base_dest)
			.map_pathless_artifact_err(ArtifactAction::Delete)?;
		artifacts::delete(&path, true).await?;

		info!("Deleted artifact file {}", path.display());
		Ok(path)
	}

	/// Deletes leftover artifacts from an old set that aren't present in a new set of artifacts
	pub async fn delete_artifacts_diff(
		&self,
		old_artifacts: impl IntoIterator<Item = &ModArtifact>,
		new_artifacts: impl IntoIterator<Item = &ModArtifact>,
	) -> crate::Result<()> {
		// Build a list of paths for both the new and old artifacts
		let new_paths = new_artifacts
			.into_iter()
			.map(|artifact| artifact.dest_within(&self.base_dest))
			.collect::<crate::Result<HashSet<PathBuf>>>()?;
		let old_paths: HashSet<PathBuf> = old_artifacts
			.into_iter()
			.map(|artifact| artifact.dest_within(&self.base_dest))
			.collect::<crate::Result<HashSet<PathBuf>>>()?;

		// Determine the paths that are no longer needed
		let unnecessary_paths: Vec<&PathBuf> = old_paths.difference(&new_paths).collect();

		// Delete each unnecessary artifact path and track any failures
		let mut failed = ArtifactErrorVec::new();
		for path in unnecessary_paths {
			if let Err(err) = artifacts::delete(path, true).await {
				failed.push(err);
			}
			info!("Deleted left-over artifact file {}", path.display());
		}

		if failed.is_empty() {
			Ok(())
		} else {
			Err(Error::Artifacts(failed))
		}
	}
}
