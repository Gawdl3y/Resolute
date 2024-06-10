use log::info;
use resolute::manager::ModManager;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

use crate::{build_http_client, settings};

/// Ensures a change to the Resonite path setting is propagated to the manager
#[tauri::command]
pub(crate) async fn resonite_path_changed(
	app: AppHandle,
	manager: State<'_, Mutex<ModManager<'_>>>,
) -> Result<(), String> {
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	manager.lock().await.set_base_dest(&resonite_path);
	info!("Changed manager's base destination to {}", resonite_path);
	Ok(())
}

/// Ensures a change to the connection timeout setting is propagated to the manager
#[tauri::command]
pub(crate) async fn connect_timeout_changed(
	app: AppHandle,
	manager: State<'_, Mutex<ModManager<'_>>>,
) -> Result<(), String> {
	let http_client = build_http_client(&app).map_err(|err| err.to_string())?;
	manager.lock().await.set_http_client(http_client);
	info!("Changed manager's HTTP client for connectTimeout setting change");
	Ok(())
}
