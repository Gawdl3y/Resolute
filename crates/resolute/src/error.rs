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
	#[error("json error: {0}")]
	Json(#[from] serde_json::Error),
}

/// Alias for a `Result` with the error type `download::Error`.
pub type Result<T> = core::result::Result<T, Error>;
