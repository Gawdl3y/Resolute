use std::path::PathBuf;

use reqwest::StatusCode;

/// Error returned from a Downloader
#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("http connection failed: {0}")]
	Request(#[from] reqwest::Error),

	#[error("http request resulted in a non-successful (not 2xx) status: {0}")]
	Http(StatusCode),

	#[error("io error: {0}")]
	Io(#[from] std::io::Error),

	#[error("unable to process path: {0}")]
	Path(String),

	#[error("unable to parse url: {0}")]
	Url(String),

	#[error("json error: {0}")]
	Json(#[from] serde_json::Error),

	#[error("checksum error for {file}: calculated hash {checksum} doesn't match expected hash {expected}")]
	Checksum {
		checksum: String,
		expected: String,
		file: String,
	},

	#[error("unknown version \"{1}\" for mod \"{0}\"")]
	UnknownVersion(String, String),

	#[error("unable to delete artifacts")]
	ArtifactDeletion(Vec<(PathBuf, std::io::Error)>),

	#[error("resonite discovery error: {0}")]
	Discovery(#[from] steamlocate::Error),

	#[cfg(feature = "db")]
	#[error("database error: {0}")]
	Database(#[from] native_db::db_type::Error),

	#[cfg(feature = "db")]
	#[error("item not found in database: {0}")]
	ItemNotFound(String),
}

/// Alias for a `Result` with the error type `resolute::Error`.
pub type Result<T> = core::result::Result<T, Error>;
