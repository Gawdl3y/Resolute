// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io, path::PathBuf, time::Duration};

use anyhow::Context;
use log::{debug, error, info, warn};
use resolute::{
	discover::discover_resonite,
	download::Downloader,
	manifest,
	mods::{self, ModVersion, ResoluteMod, ResoluteModMap},
};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, Window, WindowEvent};
use tauri_plugin_log::LogTarget;
use tauri_plugin_window_state::StateFlags;
use tokio::{fs, join, sync::Mutex};

mod settings;

fn main() -> anyhow::Result<()> {
	// Set up a shared HTTP client
	let http_client = reqwest::Client::builder()
		.connect_timeout(Duration::from_secs(10))
		.use_rustls_tls()
		.build()
		.context("Unable to build HTTP client")?;

	// Set up a shared mod downloader
	let downloader = Downloader::new(http_client.clone());

	// Set up and run the Tauri app
	tauri::Builder::default()
		.plugin(
			#[cfg(debug_assertions)]
			{
				use tauri_plugin_log::fern::colors::ColoredLevelConfig;

				tauri_plugin_log::Builder::default()
					.targets(vec![LogTarget::Stdout, LogTarget::Webview])
					.with_colors(ColoredLevelConfig::default())
					.level_for("rustls", log::LevelFilter::Debug)
					.build()
			},
			#[cfg(not(debug_assertions))]
			{
				tauri_plugin_log::Builder::default()
					.targets(vec![LogTarget::Stdout, LogTarget::LogDir])
					.level(log::LevelFilter::Debug)
					.build()
			},
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
		.invoke_handler(tauri::generate_handler![
			show_window,
			load_manifest,
			install_version,
			verify_resonite_path,
			hash_file
		])
		.manage(http_client)
		.manage(downloader)
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
		info!("Resonite path not configured, running autodiscovery");

		// Run discovery
		let resonite_dir = tauri::async_runtime::spawn_blocking(|| discover_resonite(None))
			.await
			.context("Unable to spawn blocking task for discovery")??;

		// If discovery found a path, save it to the setting
		match resonite_dir {
			Some(resonite_dir) => {
				info!("Discovered Resonite path: {}", resonite_dir.display());
				settings::set(&app, "resonitePath", resonite_dir)?
			}
			None => {
				info!("Autodiscovery didn't find a Resonite path");
			}
		}
	}

	Ok(())
}

#[tauri::command]
fn show_window(window: Window) {
	window.show().expect("unable to show main window");
}

#[tauri::command]
async fn load_manifest(app: AppHandle, bypass_cache: bool) -> Result<ResoluteModMap, String> {
	// Configure the manifest client
	let mut builder = manifest::Client::builder().cache(
		app.path_resolver()
			.app_cache_dir()
			.expect("unable to locate cache directory")
			.join("resonite-mod-manifest.json"),
	);

	// Override the manifest URL if the user has customized it
	let manifest_url: Option<String> = settings::get(&app, "manifestUrl").map_err(|err| err.to_string())?;
	if let Some(url) = manifest_url {
		builder = builder.url(url.as_ref());
	}

	// Build the manifest client using the shared HTTP client
	let http = app.state::<reqwest::Client>();
	let client = builder.http_client(http.inner().clone()).build();

	// Retrieve the manifest JSON
	let json = if !bypass_cache {
		client.retrieve().await
	} else {
		info!("Forcing download of manifest");
		client.download().await
	}
	.map_err(|err| format!("Error downloading manifest: {}", err))?;

	// Parse the manifest JSON then build a mod map out of it
	let mods = tauri::async_runtime::spawn_blocking(move || -> Result<ResoluteModMap, String> {
		let manifest = client
			.parse(json.as_str())
			.map_err(|err| format!("Error parsing manifest: {}", err))?;
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

#[tauri::command]
async fn hash_file(path: String) -> Result<String, String> {
	// Verify the path given is a file
	let meta = fs::metadata(&path)
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
		io::copy(&mut file, &mut hasher).map_err(|err| format!("Error hashing file: {}", err))?;
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

#[derive(Default)]
struct ResoluteState {
	mods: Mutex<ResoluteModMap>,
}

#[derive(Clone, serde::Serialize)]
struct Payload {
	args: Vec<String>,
	cwd: String,
}
