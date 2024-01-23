use std::{
	fmt::Display,
	path::{Path, PathBuf},
};

use tokio::fs;

use crate::Error;

/// Deletes an artifact file
pub(crate) async fn delete(path: &Path) -> Result<(), ArtifactError> {
	fs::remove_file(path).await.map_err(|err| ArtifactError {
		action: ArtifactAction::Delete,
		path: Some(path.to_owned()),
		source: Box::new(err.into()),
	})?;
	Ok(())
}

/// Renames an artifact file
pub(crate) async fn rename(from: &Path, to: &Path, ignore_nonexistent: bool) -> Result<bool, ArtifactError> {
	match fs::rename(from, to).await {
		Ok(..) => Ok(true),
		Err(err) => {
			if ignore_nonexistent && err.kind() == std::io::ErrorKind::NotFound {
				Ok(false)
			} else {
				Err(ArtifactError {
					action: ArtifactAction::Rename,
					path: Some(from.to_owned()),
					source: Box::new(err.into()),
				})
			}
		}
	}
}

/// An error performing an action on an artifact
#[derive(Debug)]
pub struct ArtifactError {
	pub action: ArtifactAction,
	pub path: Option<PathBuf>,
	pub source: Box<Error>,
}

impl ArtifactError {
	/// Creates a new ArtifactError without a path value
	pub fn new_pathless(action: ArtifactAction, source: Error) -> Self {
		Self {
			action,
			path: None,
			source: Box::new(source),
		}
	}

	/// Returns a closure that maps an [Error] to an [ArtifactError] with a None path
	#[inline]
	pub(crate) fn map_pathless(action: ArtifactAction) -> impl FnOnce(Error) -> Self {
		|err| Self::new_pathless(action, err)
	}
}

impl std::error::Error for ArtifactError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		Some(&self.source)
	}
}

impl Display for ArtifactError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.path {
			Some(path) => write!(
				f,
				"artifact {} error for file ({}): {}",
				self.action,
				path.display(),
				self.source
			),
			None => write!(f, "artifact error: {}", self.source),
		}
	}
}

/// An action being attempted on an artifact
#[derive(Debug)]
pub enum ArtifactAction {
	Download,
	Delete,
	Rename,
}

impl Display for ArtifactAction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let label = match self {
			Self::Download => "download",
			Self::Delete => "delete",
			Self::Rename => "rename",
		};

		write!(f, "{}", label)
	}
}

/// A Vec of artifact errors
#[derive(Debug, Default)]
pub struct ArtifactErrorVec(pub Vec<ArtifactError>);

impl ArtifactErrorVec {
	/// Creates a new empty error vec
	pub fn new() -> Self {
		Self(Vec::new())
	}

	#[inline]
	pub fn push(&mut self, err: ArtifactError) {
		self.0.push(err)
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.0.len()
	}
}

impl Display for ArtifactErrorVec {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = self.0.iter().map(|err| err.to_string()).collect::<Vec<_>>().join(", ");
		write!(f, "[{}]", text)
	}
}

impl std::error::Error for ArtifactErrorVec {}
