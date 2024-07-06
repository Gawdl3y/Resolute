use log::{error, info};
use resolute::{
	manager::{LoadedMods, ModManager},
	models::{ModVersion, ResoluteMod},
};
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

use crate::{build_manifest_config, settings};

/// Loads all mods from the manager
#[tauri::command]
pub(crate) async fn load_all_mods(
	app: AppHandle,
	manager: State<'_, Mutex<ModManager<'_>>>,
	bypass_cache: bool,
) -> Result<LoadedMods, String> {
	let mods = manager
		.lock()
		.await
		.get_all_mods(build_manifest_config(&app)?, bypass_cache)
		.await
		.map_err(|err| format!("Unable to get all mods from manager: {err}"))?;
	Ok(mods)
}

/// Loads installed mods from the manager
#[tauri::command]
pub(crate) async fn load_installed_mods(manager: State<'_, Mutex<ModManager<'_>>>) -> Result<LoadedMods, String> {
	let mods = manager
		.lock()
		.await
		.get_installed_mods()
		.await
		.map_err(|err| format!("Unable to get installed mods from manager: {err}"))?;
	Ok(mods)
}

/// Installs a mod version
#[tauri::command]
pub(crate) async fn install_mod_version(
	app: AppHandle,
	manager: State<'_, Mutex<ModManager<'_>>>,
	rmod: ResoluteMod,
	version: ModVersion,
) -> Result<(), String> {
	let mut manager = manager.lock().await;

	// Update the Resonite path in case the setting has changed
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	manager.set_base_dest(resonite_path);

	// Download the version
	info!("Installing mod {} v{}", rmod.name, version.semver);
	manager
		.install_mod(&rmod, version.semver.to_string(), |_, _| {})
		.await
		.map_err(|err| {
			error!("Failed to download mod {} v{}: {err}", rmod.name, version.semver);
			format!("Unable to download mod version: {err}")
		})?;

	info!("Successfully installed mod {} v{}", rmod.name, version.semver);
	Ok(())
}

/// Updates a mod to a new version
#[tauri::command]
pub(crate) async fn replace_mod_version(
	app: AppHandle,
	manager: State<'_, Mutex<ModManager<'_>>>,
	rmod: ResoluteMod,
	version: ModVersion,
) -> Result<(), String> {
	let mut manager = manager.lock().await;

	// Update the Resonite path in case the setting has changed
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	manager.set_base_dest(resonite_path);

	// Ensure the mod is installed
	let Some(old_version) = &rmod.installed_version else {
		return Err(format!(
			"Mod {} doesn't have an installed version to replace",
			rmod.name
		));
	};

	// Update the mod to the given version
	info!("Replacing mod {} v{} with v{}", rmod.name, old_version, version.semver);
	manager
		.update_mod(&rmod, version.semver.to_string(), |_, _| {})
		.await
		.map_err(|err| {
			error!(
				"Failed to replace mod {} v{} with v{}: {err}",
				rmod.name, old_version, version.semver
			);
			format!("Unable to replace mod version: {err}")
		})?;

	info!(
		"Successfully replaced mod {} v{} with v{}",
		rmod.name, old_version, version.semver
	);
	Ok(())
}

/// Uninstalls a mod
#[tauri::command]
pub(crate) async fn uninstall_mod(
	app: AppHandle,
	manager: State<'_, Mutex<ModManager<'_>>>,
	rmod: ResoluteMod,
) -> Result<(), String> {
	let mut manager = manager.lock().await;

	// Update the Resonite path in case the setting has changed
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	manager.set_base_dest(resonite_path);

	// Ensure the mod is installed
	let Some(old_version) = &rmod.installed_version else {
		return Err(format!(
			"Mod {} doesn't have an installed version to uninstall",
			rmod.name
		));
	};

	// Uninstall the mod
	info!("Uninstalling mod {} v{}", rmod.name, old_version);
	manager.uninstall_mod(&rmod).await.map_err(|err| {
		error!("Failed to uninstall mod {} v{}: {err}", rmod.name, old_version);
		format!("Unable to uninstall mod: {err}")
	})?;

	info!("Successfully uninstalled mod {} v{}", rmod.name, old_version);
	Ok(())
}
