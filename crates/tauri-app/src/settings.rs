use anyhow::{anyhow, Context, Result};
use log::error;
use serde::{de::DeserializeOwned, ser::Serialize};
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_store::{with_store, StoreCollection};

/// Retrieve a setting value from the given app's default setting store
pub(crate) fn get<T: DeserializeOwned>(app: &AppHandle, setting: &str) -> Result<Option<T>> {
	let stores = app.state::<StoreCollection<Wry>>();

	// Retrieve the setting value
	let val = with_store(app.clone(), stores, ".settings.dat", |store| {
		Ok(store.get(setting).cloned())
	})
	.map_err(|err| {
		error!("Unable to retrieve {} setting value: {}", setting, err);
		err
	})
	.with_context(|| format!("Unable to retrieve {setting} setting value"))?;

	// Deserialize the value
	match val {
		Some(val) => Ok(Some(
			serde_json::from_value(val)
				.map_err(|err| {
					error!("Unable to deserialize {setting} setting value: {err}");
					err
				})
				.with_context(|| format!("Unable to deserialize {setting} setting value"))?,
		)),
		None => Ok(None),
	}
}

/// Retrieve a setting value from the given app's default setting store, failing if there isn't a stored value
pub(crate) fn require<T: DeserializeOwned>(app: &AppHandle, setting: &str) -> Result<T> {
	get(app, setting)?.ok_or_else(|| {
		error!("Setting not configured: {}", setting);
		anyhow!("setting not configured: {}", setting)
	})
}

/// Store a setting value into the given app's default setting store
pub(crate) fn set<T: Serialize>(app: &AppHandle, setting: &str, value: T) -> Result<()> {
	let json_value = serde_json::to_value(value)?;
	let stores = app.state::<StoreCollection<Wry>>();

	with_store(app.clone(), stores, ".settings.dat", |store| {
		store.insert(setting.to_owned(), json_value).and_then(|()| store.save())
	})
	.with_context(|| format!("Unable to store {setting} setting"))
}
