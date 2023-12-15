// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io, path::PathBuf};

use anyhow::Context;
use log::{debug, error, info, warn};
use resolute::{
	download::Downloader,
	manifest,
	mods::{self, ModVersion, ResoluteMod, ResoluteModMap},
	path_discover::discover_resonite,
};
use tauri::{AppHandle, Manager, Window, WindowEvent};
use tauri_plugin_log::{fern::colors::ColoredLevelConfig, LogTarget};
use tauri_plugin_window_state::StateFlags;
use tokio::{fs, join, sync::Mutex};

mod settings;

#[cfg(debug_assertions)]
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::Webview];
#[cfg(not(debug_assertions))]
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::LogDir];

fn main() -> anyhow::Result<()> {
	tauri::Builder::default()
		.plugin({
			let mut builder = tauri_plugin_log::Builder::default().targets(LOG_TARGETS);
			#[cfg(debug_assertions)]
			{
				builder = builder.with_colors(ColoredLevelConfig::default());
			}
			builder.build()
		})
		.plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
			debug!("{}, {argv:?}, {cwd}", app.package_info().name);
			app.emit_all("single-instance", Payload { args: argv, cwd }).unwrap();
		}))
		.plugin(
			tauri_plugin_window_state::Builder::default()
				.with_state_flags(StateFlags::POSITION | StateFlags::SIZE | StateFlags::MAXIMIZED)
				.build(),
		)
		.plugin(tauri_plugin_store::Builder::default().build())
		.invoke_handler(tauri::generate_handler![
			show_window,
			load_manifest,
			install_version,
			verify_resonite_path
		])
		.manage(Downloader::default())
		.manage(ResoluteState::default())
		.setup(|app| {
			info!(
				"Resolute v{} initializing",
				app.config()
					.package
					.version
					.clone()
					.unwrap_or_else(|| "Unknown".to_owned())
			);

			let window = app.get_window("main").expect("unable to get main window");

			// Workaround for poor resize performance on Windows
			window.on_window_event(|event| {
				if let WindowEvent::Resized(..) = event {
					std::thread::sleep(std::time::Duration::from_nanos(5))
				}
			});

			// Open the dev console automatically in development
			#[cfg(debug_assertions)]
			{
				window.open_devtools();
			}

			// Create any missing app directories
			let handle = app.app_handle();
			tauri::async_runtime::spawn(async {
				if let Err(err) = create_app_dirs(handle).await {
					warn!("Unable to create some app directories: {}", err);
				}
			});

			// Discover the Resonite path if it isn't configured already
			let handle = app.app_handle();
			tauri::async_runtime::spawn(async move {
				if let Err(err) = autodiscover_resonite_path(handle).await {
					warn!("Unable to autodiscover Resonite path: {}", err);
				}
			});

			Ok(())
		})
		.run(tauri::generate_context!())
		.with_context(|| "Unable to initialize Tauri application")?;

	Ok(())
}

/// Creates any missing app directories
async fn create_app_dirs(app: AppHandle) -> Result<(), String> {
	// Create all of the directories
	let resolver = app.path_resolver();
	let results: [Result<(), io::Error>; 3] = join!(
		fs::create_dir(resolver.app_data_dir().ok_or("unable to get data dir")?),
		fs::create_dir(resolver.app_config_dir().ok_or("unable to get config dir")?),
		fs::create_dir(resolver.app_cache_dir().ok_or("unable to get cache dir")?),
	)
	.into();

	// Filter out all successful (or already existing) results
	let errors: Vec<io::Error> = results
		.into_iter()
		.filter(|res| res.is_err())
		.map(|res| res.expect_err("somehow had a non-error error when checking app dir creation for errors"))
		.filter(|err| err.kind() != io::ErrorKind::AlreadyExists)
		.collect();

	if errors.is_empty() {
		Ok(())
	} else {
		Err(errors
			.into_iter()
			.map(|err| err.to_string())
			.collect::<Vec<String>>()
			.join(", "))
	}
}

/// Auto-discovers a Resonite path if the setting isn't configured
async fn autodiscover_resonite_path(app: AppHandle) -> Result<(), anyhow::Error> {
	let path_configured = settings::get::<String>(&app, "resonitePath")?.is_some();

	// If the path isn't already configured, try to find one automatically
	if !path_configured {
		info!("Resonite path not configured, trying autodiscovery");

		let found_path = discover_resonite().await?;
		if let Some(resonite_path) = found_path {
			info!("Discovered Resonite path: {}", resonite_path.display());

			// Strip the UNC prefix from the string if it's there
			let plain = {
				let plain = resonite_path.to_str().ok_or_else(|| {
					resolute::Error::Path("unable to convert discovered resonite path to string".to_owned())
				})?;
				if plain.starts_with(r#"\\?\"#) {
					plain.strip_prefix(r#"\\?\"#).ok_or_else(|| {
						resolute::Error::Path("unable to strip unc prefix from discovered resonite path".to_owned())
					})?
				} else {
					plain
				}
			};

			settings::set(&app, "resonitePath", plain)?
		}
	}

	Ok::<(), anyhow::Error>(())
}

#[tauri::command]
fn show_window(window: Window) {
	window.show().expect("unable to show main window");
}

#[tauri::command]
async fn load_manifest(app: AppHandle, bypass_cache: bool) -> Result<ResoluteModMap, String> {
	// Build the config for all manifest operations
	let mut config = manifest::ManifestConfig::default().cache(
		app.path_resolver()
			.app_cache_dir()
			.expect("unable to locate cache directory")
			.join("resonite-mod-manifest.json"),
	);

	// Override the manifest URL if the user has customized it
	let manifest_url: Option<String> = settings::get(&app, "manifestUrl").map_err(|err| err.to_string())?;
	if let Some(url) = manifest_url {
		config = config.url(url.as_ref());
	}

	// Retrieve the manifest JSON
	let json = if !bypass_cache {
		manifest::retrieve(&config).await
	} else {
		info!("Forcing download of manifest");
		manifest::download(&config).await
	}
	.map_err(|err| format!("Error downloading manifest: {}", err))?;

	// Parse the manifest JSON then build a mod map out of it
	let mods = tauri::async_runtime::spawn_blocking(move || -> Result<ResoluteModMap, String> {
		let manifest = manifest::parse(json.as_str()).map_err(|err| format!("Error parsing manifest: {}", err))?;
		Ok(mods::load_manifest(manifest))
	})
	.await
	.map_err(|err| format!("Error loading manifest: {}", err))??;

	let state = app.state::<ResoluteState>();
	*state.mods.lock().await = mods.clone();
	Ok(mods)
}

#[tauri::command]
async fn install_version(app: AppHandle, rmod: ResoluteMod, version: ModVersion) -> Result<(), String> {
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;

	// Download the version
	let downloader = app.state::<Downloader>();
	info!("Installing mod {} v{}", rmod.name, version.semver);
	downloader
		.download_version(&version, PathBuf::from(resonite_path).as_path(), |_, _| {})
		.await
		.map_err(|err| {
			error!("Failed to download mod {} v{}: {}", rmod.name, version.semver, err);
			format!("Unable to download mod version: {}", err)
		})?;

	info!("Successfully installed mod {} v{}", rmod.name, version.semver);
	Ok(())
}

#[tauri::command]
async fn verify_resonite_path(app: AppHandle) -> Result<bool, String> {
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	tokio::fs::try_exists(resonite_path)
		.await
		.map_err(|err| err.to_string())
}

#[derive(Default)]
struct ResoluteState {
	mods: Mutex<ResoluteModMap>,
}

#[derive(Clone, serde::Serialize)]
struct Payload {
	args: Vec<String>,
	cwd: String,
}
