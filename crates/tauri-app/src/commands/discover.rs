use log::{error, info};
use resolute::{discover, manager::ModManager, mods::ResoluteModMap};
use tauri::AppHandle;
use tokio::sync::Mutex;

use crate::{build_manifest_config, settings};

/// Looks for a possible Resonite path
#[tauri::command]
pub(crate) async fn discover_resonite_path() -> Result<Option<String>, String> {
	let path = tauri::async_runtime::spawn_blocking(|| discover::discover_resonite(None))
		.await
		.map_err(|err| {
			error!("Unable to spawn blocking task for Resonite path discovery: {}", err);
			format!("Unable to spawn blocking task for Resonite path discovery: {}", err)
		})?
		.map_err(|err| {
			error!("Unable to discover Resonite path: {}", err);
			format!("Unable to discover Resonite path: {}", err)
		})?;

	match path {
		Some(path) => path.to_str().map(|path| Some(path.to_owned())).ok_or_else(|| {
			error!("Unable to convert discovered Resonite path ({:?}) to a String", path);
			"Unable to convert discovered Resonite path to a String".to_owned()
		}),
		None => Ok(None),
	}
}

/// Discovers installed mods
#[tauri::command]
pub(crate) async fn discover_installed_mods(
	app: AppHandle,
	manager: tauri::State<'_, Mutex<ModManager<'_>>>,
) -> Result<ResoluteModMap, String> {
	let mut manager = manager.lock().await;

	// Update the Resonite path in case the setting has changed
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	manager.set_base_dest(resonite_path);

	// Run the discovery
	info!("Discovering installed mods");
	let mods = manager
		.discover_installed_mods(build_manifest_config(&app)?)
		.await
		.map_err(|err| {
			error!("Unable to discover installed mods: {}", err);
			format!("Unable to discover installed mods: {}", err)
		})?;

	Ok(mods)
}
