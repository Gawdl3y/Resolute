// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io;

use anyhow::Context;
use log::{debug, warn};
use resolute::{
	manifest,
	mods::{self, ResoluteModMap},
};
use tauri::{Manager, WindowEvent};
use tauri_plugin_log::{fern::colors::ColoredLevelConfig, LogTarget};
use tauri_plugin_window_state::StateFlags;
use tokio::{fs, join};

#[cfg(debug_assertions)]
const LOG_TARGETS: [LogTarget; 3] = [LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview];
#[cfg(not(debug_assertions))]
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::LogDir, LogTarget::Stdout];

fn main() -> anyhow::Result<()> {
	tauri::Builder::default()
		.plugin(
			tauri_plugin_log::Builder::default()
				.targets(LOG_TARGETS)
				.with_colors(ColoredLevelConfig::default())
				.build(),
		)
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
		.invoke_handler(tauri::generate_handler![show_window, load_manifest])
		.setup(|app| {
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

			Ok(())
		})
		.run(tauri::generate_context!())
		.with_context(|| "Unable to initialize Tauri application")?;

	Ok(())
}

/// Creates any missing app directories
async fn create_app_dirs(app: tauri::AppHandle) -> Result<(), String> {
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

#[tauri::command]
fn show_window(window: tauri::Window) {
	window.show().expect("unable to show main window");
}

#[tauri::command]
async fn load_manifest(app: tauri::AppHandle, bypass_cache: bool) -> Result<ResoluteModMap, String> {
	// Build the config for all manifest operations
	let config = manifest::ManifestConfig::default().cache(
		app.path_resolver()
			.app_cache_dir()
			.expect("unable to locate cache directory")
			.join("resonite-mod-manifest.json"),
	);

	// Retrieve the manifest JSON
	let json = if !bypass_cache {
		manifest::retrieve(&config).await
	} else {
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

	Ok(mods)
}

#[derive(Clone, serde::Serialize)]
struct Payload {
	args: Vec<String>,
	cwd: String,
}
