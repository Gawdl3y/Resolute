// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io, time::Duration};

use anyhow::{anyhow, Context};
use itertools::Itertools;
use log::{debug, error, info, warn};
use native_db::DatabaseBuilder;
use once_cell::sync::Lazy;
use resolute::{
	db::ResoluteDatabase,
	discover,
	manager::{LoadedMods, ModManager},
	manifest,
	mods::{ModVersion, ResoluteMod, ResoluteModMap},
};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, Window, WindowEvent};
use tauri_plugin_log::LogTarget;
use tauri_plugin_window_state::StateFlags;
use tokio::{fs, io::AsyncReadExt, join, sync::Mutex};

mod settings;

static mut DB_BUILDER: Lazy<DatabaseBuilder> = Lazy::new(DatabaseBuilder::new);

fn main() -> anyhow::Result<()> {
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
				use tauri_plugin_log::RotationStrategy;

				tauri_plugin_log::Builder::default()
					.targets(vec![LogTarget::Stdout, LogTarget::Webview, LogTarget::LogDir])
					.rotation_strategy(RotationStrategy::KeepAll)
					.max_file_size(1024 * 256)
					.level(log::LevelFilter::Debug)
					.level_for(
						"tao::platform_impl::platform::event_loop::runner",
						log::LevelFilter::Error,
					)
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
			load_all_mods,
			load_installed_mods,
			install_mod_version,
			replace_mod_version,
			uninstall_mod,
			discover_resonite_path,
			discover_installed_mods,
			verify_resonite_path,
			hash_file,
			get_session_log,
			open_log_dir,
			resonite_path_changed,
			connect_timeout_changed,
		])
		.setup(|app| {
			let window = app.get_window("main").ok_or("unable to get main window")?;

			// Workaround for poor resize performance on Windows
			window.on_window_event(|event| {
				if let WindowEvent::Resized(..) = event {
					std::thread::sleep(std::time::Duration::from_nanos(5))
				}
			});

			// Open the dev console automatically in development
			#[cfg(debug_assertions)]
			window.open_devtools();

			// Initialize the app
			let handle = app.app_handle();
			tauri::async_runtime::spawn(async move {
				if let Err(err) = init(handle.clone()).await {
					error!("Initialization failed: {}", err);
					build_error_window(handle, err);
				}
			});

			Ok(())
		})
		.run(tauri::generate_context!())
		.with_context(|| "Unable to initialize Tauri application")?;

	Ok(())
}

/// Initializes the app
async fn init(app: AppHandle) -> Result<(), anyhow::Error> {
	info!(
		"Resolute v{} initializing",
		app.config()
			.package
			.version
			.clone()
			.unwrap_or_else(|| "Unknown".to_owned())
	);

	// Ensure all needed app directories are created
	if let Err(err) = create_app_dirs(app.clone()).await {
		warn!("Unable to create some app directories: {}", err);
	}

	// Discover the Resonite path in the background if it isn't configured already
	let handle = app.app_handle();
	tauri::async_runtime::spawn(async {
		if let Err(err) = autodiscover_resonite_path(handle).await {
			warn!("Unable to autodiscover Resonite path: {}", err);
		}
	});

	tauri::async_runtime::spawn_blocking(move || {
		// Open the database
		let resolver = app.path_resolver();
		let db_path = resolver
			.app_data_dir()
			.ok_or_else(|| anyhow!("Unable to get data dir"))?
			.join("resolute.db");
		info!("Opening database at {}", db_path.display());
		let db = unsafe { ResoluteDatabase::open(&mut DB_BUILDER, db_path) }.context("Unable to open database")?;

		// Get the Resonite path setting
		info!("Retrieving Resonite path from settings store");
		let resonite_path = settings::get(&app, "resonitePath")
			.context("Unable to get resonitePath setting")?
			.unwrap_or_else(|| "".to_owned());
		info!("Resonite path: {}", &resonite_path);

		// Set up the shared mod manager
		info!("Setting up mod manager");
		let http_client = build_http_client(&app)?;
		let manager = ModManager::new(db, resonite_path, http_client);
		app.manage(Mutex::new(manager));

		Ok::<(), anyhow::Error>(())
	})
	.await
	.context("Error running blocking task for initialization")??;

	info!("Resolute initialized");
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
		let resonite_dir = tauri::async_runtime::spawn_blocking(|| discover::discover_resonite(None))
			.await
			.context("Unable to spawn blocking task for Resonite path autodiscovery")??;

		// If discovery found a path, save it to the setting
		match resonite_dir {
			Some(resonite_dir) => {
				info!("Discovered Resonite path: {}", resonite_dir.display());
				settings::set(&app, "resonitePath", &resonite_dir)?;

				if let Some(manager) = app.try_state::<Mutex<ModManager>>() {
					manager.lock().await.set_base_dest(resonite_dir);
				}
			}
			None => {
				info!("Autodiscovery didn't find a Resonite path");
			}
		}
	}

	Ok(())
}

/// Builds a manifest config that takes the user-configured settings into account
fn build_manifest_config(app: &AppHandle) -> Result<manifest::Config, String> {
	// Build the base config
	let mut config = manifest::Config {
		cache_file_path: Some(
			app.path_resolver()
				.app_cache_dir()
				.ok_or_else(|| "Unable to locate cache directory".to_owned())?
				.join("resonite-mod-manifest.json"),
		),
		..manifest::Config::default()
	};

	// Override the manifest URL if the user has configured a custom one
	let manifest_url: Option<String> = settings::get(app, "manifestUrl").map_err(|err| err.to_string())?;
	if let Some(url) = manifest_url {
		config
			.set_remote_url(url.as_ref())
			.map_err(|_err| "Unable to parse custom manifest URL".to_owned())?;
	}

	Ok(config)
}

/// Builds an HTTP client that takes the user-configured settings into account
fn build_http_client(app: &AppHandle) -> Result<reqwest::Client, anyhow::Error> {
	let connect_timeout: f32 = settings::get(app, "connectTimeout")?.unwrap_or(10f32);
	debug!("Building HTTP client, connectTimeout = {}s", connect_timeout);

	reqwest::Client::builder()
		.connect_timeout(Duration::from_secs_f32(connect_timeout))
		.use_rustls_tls()
		.build()
		.context("Unable to build HTTP client")
}

/// Builds the error window for a given error, then closes the main window
fn build_error_window(app: AppHandle, err: anyhow::Error) {
	let init_script = format!("globalThis.error = `{:?}`;", err);
	tauri::WindowBuilder::new(&app, "error", tauri::WindowUrl::App("error.html".into()))
		.title("Resolute")
		.center()
		.resizable(false)
		.visible(false)
		.initialization_script(&init_script)
		.build()
		.expect("Error occurred while initializing and the error window couldn't be displayed");
	let _ = app.get_window("main").expect("unable to get main window").close();
}

/// Sets the requesting window's visibility to shown
#[tauri::command]
fn show_window(window: Window) -> Result<(), String> {
	window.show().map_err(|err| format!("Unable to show window: {}", err))?;
	Ok(())
}

/// Loads all mods from the manager
#[tauri::command]
async fn load_all_mods(
	app: AppHandle,
	manager: tauri::State<'_, Mutex<ModManager<'_>>>,
	bypass_cache: bool,
) -> Result<LoadedMods, String> {
	let mods = manager
		.lock()
		.await
		.get_all_mods(build_manifest_config(&app)?, bypass_cache)
		.await
		.map_err(|err| format!("Unable to get all mods from manager: {}", err))?;
	Ok(mods)
}

/// Loads installed mods from the manager
#[tauri::command]
async fn load_installed_mods(manager: tauri::State<'_, Mutex<ModManager<'_>>>) -> Result<LoadedMods, String> {
	let mods = manager
		.lock()
		.await
		.get_installed_mods()
		.await
		.map_err(|err| format!("Unable to get installed mods from manager: {}", err))?;
	Ok(mods)
}

/// Installs a mod version
#[tauri::command]
async fn install_mod_version(
	app: AppHandle,
	manager: tauri::State<'_, Mutex<ModManager<'_>>>,
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
			error!("Failed to download mod {} v{}: {}", rmod.name, version.semver, err);
			format!("Unable to download mod version: {}", err)
		})?;

	info!("Successfully installed mod {} v{}", rmod.name, version.semver);
	Ok(())
}

/// Updates a mod to a new version
#[tauri::command]
async fn replace_mod_version(
	app: AppHandle,
	manager: tauri::State<'_, Mutex<ModManager<'_>>>,
	rmod: ResoluteMod,
	version: ModVersion,
) -> Result<(), String> {
	let mut manager = manager.lock().await;

	// Update the Resonite path in case the setting has changed
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	manager.set_base_dest(resonite_path);

	// Ensure the mod is installed
	let old_version = match &rmod.installed_version {
		Some(version) => version,
		None => {
			return Err(format!(
				"Mod {} doesn't have an installed version to replace",
				rmod.name
			))
		}
	};

	// Update the mod to the given version
	info!("Replacing mod {} v{} with v{}", rmod.name, old_version, version.semver);
	manager
		.update_mod(&rmod, version.semver.to_string(), |_, _| {})
		.await
		.map_err(|err| {
			error!(
				"Failed to replace mod {} v{} with v{}: {}",
				rmod.name, old_version, version.semver, err
			);
			format!("Unable to replace mod version: {}", err)
		})?;

	info!(
		"Successfully replaced mod {} v{} with v{}",
		rmod.name, old_version, version.semver
	);
	Ok(())
}

/// Uninstalls a mod
#[tauri::command]
async fn uninstall_mod(
	app: AppHandle,
	manager: tauri::State<'_, Mutex<ModManager<'_>>>,
	rmod: ResoluteMod,
) -> Result<(), String> {
	let mut manager = manager.lock().await;

	// Update the Resonite path in case the setting has changed
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	manager.set_base_dest(resonite_path);

	// Ensure the mod is installed
	let old_version = match &rmod.installed_version {
		Some(version) => version,
		None => {
			return Err(format!(
				"Mod {} doesn't have an installed version to uninstall",
				rmod.name
			))
		}
	};

	// Uninstall the mod
	info!("Uninstalling mod {} v{}", rmod.name, old_version);
	manager.uninstall_mod(&rmod).await.map_err(|err| {
		error!("Failed to uninstall mod {} v{}: {}", rmod.name, old_version, err);
		format!("Unable to uninstall mod: {}", err)
	})?;

	info!("Successfully uninstalled mod {} v{}", rmod.name, old_version);
	Ok(())
}

/// Looks for a possible Resonite path
#[tauri::command]
async fn discover_resonite_path() -> Result<Option<String>, String> {
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
async fn discover_installed_mods(
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

/// Verifies the Resonite path specified in the settings store exists
#[tauri::command]
async fn verify_resonite_path(app: AppHandle) -> Result<bool, String> {
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	tokio::fs::try_exists(resonite_path)
		.await
		.map_err(|err| err.to_string())
}

/// Calculates the SHA-256 checksum of a file
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

/// Gets the log file content from this session
#[tauri::command]
async fn get_session_log(app: AppHandle) -> Result<String, String> {
	// Figure out the path to the log file
	let resolver = app.path_resolver();
	let mut log_path = resolver.app_log_dir().ok_or("Unable to get log directory")?;
	log_path.push(format!("{}.log", app.package_info().name));

	let log = {
		// Open and read the file
		let mut file = fs::File::open(log_path)
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
async fn open_log_dir(app: AppHandle) -> Result<(), String> {
	let path = app
		.path_resolver()
		.app_log_dir()
		.ok_or_else(|| "Unable to get log directory".to_owned())?;
	opener::open(path).map_err(|err| format!("Unable to open log directory: {}", err))?;
	Ok(())
}

/// Ensures a change to the Resonite path setting is propagated to the manager
#[tauri::command]
async fn resonite_path_changed(app: AppHandle, manager: tauri::State<'_, Mutex<ModManager<'_>>>) -> Result<(), String> {
	let resonite_path: String = settings::require(&app, "resonitePath").map_err(|err| err.to_string())?;
	manager.lock().await.set_base_dest(&resonite_path);
	info!("Changed manager's base destination to {}", resonite_path);
	Ok(())
}

/// Ensures a change to the connection timeout setting is propagated to the manager
#[tauri::command]
async fn connect_timeout_changed(
	app: AppHandle,
	manager: tauri::State<'_, Mutex<ModManager<'_>>>,
) -> Result<(), String> {
	let http_client = build_http_client(&app).map_err(|err| err.to_string())?;
	manager.lock().await.set_http_client(http_client);
	info!("Changed manager's HTTP client for connectTimeout setting change");
	Ok(())
}

#[derive(Clone, serde::Serialize)]
struct Payload {
	args: Vec<String>,
	cwd: String,
}
