use std::{fmt::Display, path::PathBuf};

use reqwest::StatusCode;
use semver::Version;

use crate::mods::ResoluteMod;

/// Error returned from a Downloader
#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("http connection failed: {0}")]
	Request(#[from] reqwest::Error),

	#[error("http request resulted in a non-successful (not 2xx) status: {0}")]
	Http(StatusCode),

	#[error("io error: {0}")]
	Io(#[from] std::io::Error),

	#[error("task error: {0}")]
	Task(#[from] tokio::task::JoinError),

	#[error("unable to process path: {0}")]
	Path(String),

	#[error("unable to parse url: {0}")]
	Url(String),

	#[error("unable to parse semver: {0}")]
	Semver(#[from] semver::Error),

	#[error("json error: {0}")]
	Json(#[from] serde_json::Error),

	#[error("checksum error for {file}: calculated hash {checksum} doesn't match expected hash {expected}")]
	Checksum {
		checksum: String,
		expected: String,
		file: String,
	},

	#[error("unknown version \"{1}\" for mod \"{0}\"")]
	UnknownVersion(String, Version),

	#[error("mod \"{0}\" isn't installed")]
	ModNotInstalled(Box<ResoluteMod>),

	#[error("artifact error: {0}")]
	Artifact(ArtifactError),

	#[error("multiple artifact errors: {0}")]
	Artifacts(ArtifactErrorVec),

	#[error("resonite discovery error: {0}")]
	Discovery(#[from] steamlocate::Error),

	#[cfg(feature = "db")]
	#[error("database error: {0}")]
	Database(#[from] native_db::db_type::Error),

	#[cfg(feature = "db")]
	#[error("item not found in database: {0}")]
	ItemNotFound(String),
}

/// An error performing an action on an artifact
#[derive(Debug)]
pub struct ArtifactError {
	pub action: ArtifactAction,
	pub path: Option<PathBuf>,
	pub source: Box<Error>,
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

/// Alias for a `Result` with the error type `resolute::Error`.
pub type Result<T> = core::result::Result<T, Error>;
