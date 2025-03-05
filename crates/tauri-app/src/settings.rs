use anyhow::{anyhow, Context, Result};
use log::error;
use serde::{de::DeserializeOwned, ser::Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

/// Retrieve a setting value from the given app's default setting store
pub(crate) fn get<T: DeserializeOwned>(app: &AppHandle, setting: &str) -> Result<Option<T>> {
	let store = app
		.store(".settings.dat")
		.inspect_err(|err| error!("Unable to open settings store to read {setting}: {err}"))
		.context("Unable to open settings store")?;

	// Retrieve and deserialize the value
	if let Some(val) = store.get(setting) {
		Ok(Some(
			serde_json::from_value(val)
				.inspect_err(|err| error!("Unable to deserialize {setting} setting value: {err}"))
				.with_context(|| format!("Unable to deserialize {setting} setting value"))?,
		))
	} else {
		Ok(None)
	}
}

/// Retrieve a setting value from the given app's default setting store, failing if there isn't a stored value
pub(crate) fn require<T: DeserializeOwned>(app: &AppHandle, setting: &str) -> Result<T> {
	get(app, setting)?.ok_or_else(|| {
		error!("Setting not configured: {}", setting);
		anyhow!("Setting not configured: {}", setting)
	})
}

/// Store a setting value into the given app's default setting store
pub(crate) fn set<T: Serialize>(app: &AppHandle, setting: &str, value: T) -> Result<()> {
	let json_value = serde_json::to_value(value)?;
	let store = app
		.store(".settings.dat")
		.inspect_err(|err| error!("Unable to open settings store to write {setting}: {err}"))
		.context("Unable to open settings store")?;

	store.set(setting, json_value);
	store
		.save()
		.inspect_err(|err| error!("Unable to save settings store after writing {setting}: {err}"))
		.context("Unable to save settings store")
}
