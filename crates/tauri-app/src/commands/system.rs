use itertools::Itertools;
use log::{error, info};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, Window};
use tokio::io::AsyncReadExt;

use crate::settings;

/// Sets the requesting window's visibility to shown
#[tauri::command]
pub(crate) fn show_window(window: Window) -> Result<(), String> {
	window.show().map_err(|err| format!("Unable to show window: {}", err))?;
	Ok(())
}

/// Gets the Tauri app's bundle identifier
#[tauri::command]
pub(crate) fn get_app_info(app: AppHandle) -> Result<AppInfo, String> {
	let config = app.config();
	Ok(AppInfo {
		name: config
			.product_name
			.clone()
			.ok_or_else(|| "Unable to get app name".to_owned())?,
		identifier: config.identifier.clone(),
		version: config
			.version
			.clone()
			.ok_or_else(|| "Unable to get app version".to_owned())?,
		tauri_version: tauri::VERSION.to_owned(),
		debug: cfg!(debug_assertions),
	})
}

/// Verifies the Resonite path specified in the settings store exists
#[tauri::command]
pub(crate) async fn verify_resonite_path(app: AppHandle) -> Result<bool, String> {
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	tokio::fs::try_exists(resonite_path)
		.await
		.map_err(|err| err.to_string())
}

/// Calculates the SHA-256 checksum of a file
#[tauri::command]
pub(crate) async fn hash_file(path: String) -> Result<String, String> {
	// Verify the path given is a file
	let meta = tokio::fs::metadata(&path)
		.await
		.map_err(|err| format!("Unable to read metadata of path: {}", err))?;
	if !meta.is_file() {
		return Err("The supplied path isn't a file. Hashing of directories isn't supported.".to_owned());
	}

	// Hash the file
	info!("Hashing file {}", path);
	let file = path.clone();
	let digest = tauri::async_runtime::spawn_blocking(move || {
		let mut hasher = Sha256::new();
		let mut file = std::fs::File::open(file).map_err(|err| format!("Error opening file: {}", err))?;
		std::io::copy(&mut file, &mut hasher).map_err(|err| format!("Error hashing file: {}", err))?;
		Ok::<_, String>(hasher.finalize())
	})
	.await
	.map_err(|err| {
		error!("Error spawning hashing task for file {}: {}", path, err);
		format!("Error spawning hashing task: {}", err)
	})?
	.map_err(|err| {
		error!("Error hashing file {}: {}", path, err);
		format!("Error hashing file: {}", err)
	})?;

	let hash = format!("{:x}", digest);
	info!("Finished hashing file {}: {}", path, hash);
	Ok(hash)
}

/// Gets the log file content from this session
#[tauri::command]
pub(crate) async fn get_session_log(app: AppHandle) -> Result<String, String> {
	// Figure out the path to the log file
	let resolver = app.path();
	let mut log_path = resolver
		.app_log_dir()
		.map_err(|err| format!("Unable to get log directory: {}", err))?;
	log_path.push(format!("{}.log", app.package_info().name));

	let log = {
		// Open and read the file
		let mut file = tokio::fs::File::open(log_path)
			.await
			.map_err(|err| format!("Error opening log file: {}", err))?;
		let mut log = String::new();
		file.read_to_string(&mut log)
			.await
			.map_err(|err| format!("Error reading log file contents: {}", err))?;

		// Get only the log lines after the most recent initializing line
		log.lines()
			.rev()
			.take_while_inclusive(|line| !line.ends_with("initializing"))
			.fold(String::new(), |mut acc, line| {
				acc.insert(0, '\n');
				acc.insert_str(0, line);
				acc
			})
	};

	Ok(log)
}

/// Opens the app's log directory in the system file browser
#[tauri::command]
pub(crate) async fn open_log_dir(app: AppHandle) -> Result<(), String> {
	let path = app
		.path()
		.app_log_dir()
		.map_err(|err| format!("Unable to get log directory: {}", err))?;
	opener::open(path).map_err(|err| format!("Unable to open log directory: {}", err))?;
	Ok(())
}

/// Information about the app
#[derive(Serialize, Deserialize)]
pub(crate) struct AppInfo {
	pub name: String,
	pub identifier: String,
	pub version: String,
	pub tauri_version: String,
	pub debug: bool,
}
