use core::result;
use std::io;

#[cfg(feature = "db")]
use native_db::db_type;
use reqwest::StatusCode;
use semver::Version;
use tokio::task;

use crate::{
	manager::artifacts::{ArtifactError, ArtifactErrorVec},
	models::ResoluteMod,
};

/// Error returned from a Downloader
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
	#[error("http connection failed: {0}")]
	Request(#[from] reqwest::Error),

	#[error("http request resulted in a non-successful (not 2xx) status: {0}")]
	Http(StatusCode),

	#[error("io error: {0}")]
	Io(#[from] io::Error),

	#[error("task error: {0}")]
	Task(#[from] task::JoinError),

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
	Artifact(#[from] ArtifactError),

	#[error("multiple artifact errors: {0}")]
	Artifacts(#[from] ArtifactErrorVec),

	#[error("no old artifact exists to delete")]
	NoOldArtifact,

	#[error("resonite discovery error: {0}")]
	Discovery(#[from] steamlocate::Error),

	#[cfg(feature = "db")]
	#[error("database error: {0}")]
	Database(#[from] db_type::Error),

	#[cfg(feature = "db")]
	#[error("item not found in database: {0}")]
	ItemNotFound(String),
}

/// Alias for a `Result` with the error type `resolute::Error`.
pub type Result<T> = result::Result<T, Error>;
