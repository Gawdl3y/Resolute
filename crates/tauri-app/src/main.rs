// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io, time::Duration};

use anyhow::{anyhow, Context};
use log::{debug, error, info, warn};
use native_db::DatabaseBuilder;
use once_cell::sync::Lazy;
use resolute::{db::ResoluteDatabase, discover, manager::ModManager, manifest};
use tauri::{AppHandle, Manager, WindowEvent};
use tauri_plugin_log::LogTarget;
use tauri_plugin_window_state::StateFlags;
use tokio::{fs, join, sync::Mutex};

mod commands;
mod settings;

/// Lazily-initialized database builder for the Resolute DB
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
			commands::manager::load_all_mods,
			commands::manager::load_installed_mods,
			commands::manager::install_mod_version,
			commands::manager::replace_mod_version,
			commands::manager::uninstall_mod,
			commands::discover::discover_resonite_path,
			commands::discover::discover_installed_mods,
			commands::system::show_window,
			commands::system::verify_resonite_path,
			commands::system::hash_file,
			commands::system::get_session_log,
			commands::system::open_log_dir,
			commands::settings::resonite_path_changed,
			commands::settings::connect_timeout_changed,
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
		.run(
			#[cfg(debug_assertions)]
			{
				let mut context = tauri::generate_context!();
				context.config_mut().tauri.bundle.identifier += ".debug";
				context
			},
			#[cfg(not(debug_assertions))]
			{
				tauri::generate_context!()
			},
		)
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

/// Builds a manifest config that takes the user-configured settings into account
pub(crate) fn build_manifest_config(app: &AppHandle) -> Result<manifest::Config, String> {
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
pub(crate) fn build_http_client(app: &AppHandle) -> Result<reqwest::Client, anyhow::Error> {
	let connect_timeout: f32 = settings::get(app, "connectTimeout")?.unwrap_or(10f32);
	debug!("Building HTTP client, connectTimeout = {}s", connect_timeout);

	reqwest::Client::builder()
		.connect_timeout(Duration::from_secs_f32(connect_timeout))
		.use_rustls_tls()
		.build()
		.context("Unable to build HTTP client")
}

/// Payload for a single-instance event
#[derive(Clone, serde::Serialize)]
struct Payload {
	args: Vec<String>,
	cwd: String,
}
