// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(macro_use_extern_crate, meta_variable_misuse, unit_bindings)]
#![warn(
	explicit_outlives_requirements,
	// missing_docs,
	missing_debug_implementations,
	single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_crate_dependencies,
	unused_import_braces,
	unused_lifetimes,
	unused_qualifications,
	variant_size_differences,
	clippy::pedantic,
	clippy::absolute_paths,
	clippy::arithmetic_side_effects,
	clippy::clone_on_ref_ptr,
	clippy::cognitive_complexity,
	clippy::create_dir,
	clippy::dbg_macro,
	clippy::empty_enum_variants_with_brackets,
	clippy::empty_structs_with_brackets,
	clippy::exhaustive_enums,
	clippy::exhaustive_structs,
	clippy::filetype_is_file,
	clippy::float_cmp_const,
	clippy::missing_const_for_fn,
	clippy::fn_to_numeric_cast_any,
	clippy::format_push_string,
	clippy::get_unwrap,
	clippy::if_then_some_else_none,
	clippy::infinite_loop,
	clippy::integer_division,
	clippy::lossy_float_literal,
	clippy::map_err_ignore,
	// clippy::missing_docs_in_private_items,
	clippy::mixed_read_write_in_expression,
	clippy::multiple_inherent_impl,
	clippy::mutex_atomic,
	clippy::negative_feature_names,
	clippy::panic_in_result_fn,
	clippy::print_stderr,
	clippy::print_stdout,
	clippy::pub_without_shorthand,
	clippy::rc_buffer,
	clippy::rc_mutex,
	clippy::redundant_feature_names,
	clippy::redundant_type_annotations,
	clippy::ref_patterns,
	clippy::rest_pat_in_fully_bound_structs,
	clippy::same_name_method,
	clippy::self_named_module_files,
	clippy::semicolon_inside_block,
	clippy::str_to_string,
	clippy::string_lit_chars_any,
	clippy::string_to_string,
	clippy::suspicious_xor_used_as_pow,
	clippy::tests_outside_test_module,
	clippy::try_err,
	clippy::undocumented_unsafe_blocks,
	clippy::unnecessary_safety_comment,
	clippy::unnecessary_safety_doc,
	clippy::unnecessary_self_imports,
	clippy::unneeded_field_pattern,
	clippy::unwrap_in_result,
	clippy::unwrap_used,
	clippy::verbose_file_reads,
	clippy::wildcard_dependencies,
)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::{env, error::Error, io, thread, time::Duration};

use anyhow::{anyhow, Context};
use log::{debug, error, info, warn, LevelFilter};
use native_db::DatabaseBuilder;
use once_cell::sync::Lazy;
use resolute::{db::ResoluteDatabase, discover, manager::ModManager, manifest};
use tauri::{async_runtime, App, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_window_state::StateFlags;
use tokio::{fs, join, sync::Mutex};

mod commands;
mod settings;

/// Lazily-initialized database builder for the Resolute DB
static DB_BUILDER: Lazy<DatabaseBuilder> = Lazy::new(|| {
	let mut builder = DatabaseBuilder::new();
	ResoluteDatabase::define_models(&mut builder).expect("unable to define models on database builder");
	builder
});

fn main() -> anyhow::Result<()> {
	// Set up and run the Tauri app
	tauri::Builder::default()
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_fs::init())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_process::init())
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_updater::Builder::new().build())
		.plugin(tauri_plugin_store::Builder::default().build())
		.plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
			debug!("{}, {argv:?}, {cwd}", app.package_info().name);
			app.emit("single-instance", Payload { args: argv, cwd })
				.expect("Unable to emit single-instance event with payload");
		}))
		.plugin(
			tauri_plugin_window_state::Builder::default()
				.with_state_flags(StateFlags::POSITION | StateFlags::SIZE | StateFlags::MAXIMIZED)
				.build(),
		)
		.plugin(
			#[cfg(debug_assertions)]
			{
				tauri_plugin_log::Builder::default()
					.targets([
						Target::new(TargetKind::Stdout),
						Target::new(TargetKind::Webview),
						Target::new(TargetKind::LogDir { file_name: None }),
					])
					.max_file_size(1024 * 1024)
					.level_for("rustls", LevelFilter::Debug)
					.build()
			},
			#[cfg(not(debug_assertions))]
			{
				use tauri_plugin_log::RotationStrategy;

				tauri_plugin_log::Builder::default()
					.targets([
						Target::new(TargetKind::Stdout),
						Target::new(TargetKind::Webview),
						Target::new(TargetKind::LogDir { file_name: None }),
					])
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
		.invoke_handler(tauri::generate_handler![
			commands::manager::load_all_mods,
			commands::manager::load_installed_mods,
			commands::manager::install_mod_version,
			commands::manager::replace_mod_version,
			commands::manager::uninstall_mod,
			commands::discover::discover_resonite_path,
			commands::discover::discover_installed_mods,
			commands::system::show_window,
			commands::system::get_app_info,
			commands::system::verify_resonite_path,
			commands::system::hash_file,
			commands::system::get_session_log,
			commands::system::open_resonite_dir,
			commands::system::open_log_dir,
			commands::settings::resonite_path_changed,
			commands::settings::connect_timeout_changed,
		])
		.setup(setup)
		.run(
			#[cfg(debug_assertions)]
			{
				#[allow(clippy::str_to_string)]
				let mut context = tauri::generate_context!();
				context.config_mut().identifier += ".debug";
				context
			},
			#[cfg(not(debug_assertions))]
			{
				tauri::generate_context!()
			},
		)
		.with_context(|| "Unable to initialize Tauri application")
}

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
	let window = app.get_webview_window("main").ok_or("unable to get main window")?;

	// Workaround for poor resize performance on Windows
	window.on_window_event(|event| {
		if let WindowEvent::Resized(..) = event {
			thread::sleep(Duration::from_nanos(5));
		}
	});

	// Rename the window and open the dev console in development
	#[cfg(debug_assertions)]
	{
		let mut title = window.title()?;
		title.push_str(" (debug)");
		window.set_title(&title)?;

		window.open_devtools();
	}

	// Initialize the app
	let handle = app.app_handle().clone();
	async_runtime::spawn(async move {
		if let Err(err) = init(&handle).await {
			error!("Initialization failed: {}", err);
			build_error_window(&handle, err);
		}
	});

	Ok(())
}

/// Initializes the app
async fn init(app: &AppHandle) -> Result<(), anyhow::Error> {
	let config = app.config();
	info!(
		"Resolute v{} initializing",
		config.version.clone().unwrap_or_else(|| "Unknown".to_owned())
	);

	#[cfg(debug_assertions)]
	warn!("App is in debug mode");

	debug!("Operating system: {} {}", env::consts::OS, env::consts::ARCH);
	debug!("Tauri version: {}", tauri::VERSION);

	// Ensure all needed app directories are created
	if let Err(err) = create_app_dirs(app).await {
		warn!("Unable to create some app directories: {}", err);
	}

	// Discover the Resonite path in the background if it isn't configured already
	let handle = app.clone();
	async_runtime::spawn(async move {
		if let Err(err) = autodiscover_resonite_path(&handle).await {
			warn!("Unable to autodiscover Resonite path: {}", err);
		}
	});

	let handle = app.clone();
	async_runtime::spawn_blocking(move || {
		// Open the database
		let resolver = handle.path();
		let db_path = resolver
			.app_data_dir()
			.context("Unable to get data dir")?
			.join("resolute.db");
		let db = ResoluteDatabase::open(&DB_BUILDER, db_path).context("Unable to open database")?;

		// Get the Resonite path setting
		info!("Retrieving Resonite path from settings store");
		let resonite_path: String = settings::get(&handle, "resonitePath")
			.context("Unable to get resonitePath setting")?
			.unwrap_or_default();
		info!("Resonite path: {}", &resonite_path);

		// Set up the shared mod manager
		info!("Setting up mod manager");
		let http_client = build_http_client(&handle)?;
		let manager = ModManager::new(db, resonite_path, http_client);
		handle.manage(Mutex::new(manager));

		Ok::<(), anyhow::Error>(())
	})
	.await
	.context("Error running blocking task for initialization")??;

	info!("Resolute initialized");
	Ok(())
}

/// Creates any missing app directories
async fn create_app_dirs(app: &AppHandle) -> Result<(), String> {
	// Create all of the directories
	let resolver = app.path();
	let results: [Result<(), io::Error>; 3] = join!(
		fs::create_dir(
			resolver
				.app_data_dir()
				.map_err(|err| format!("unable to get data dir: {err}"))?
		),
		fs::create_dir(
			resolver
				.app_config_dir()
				.map_err(|err| format!("unable to get config dir: {err}"))?
		),
		fs::create_dir(
			resolver
				.app_cache_dir()
				.map_err(|err| format!("unable to get cache dir: {err}"))?
		),
	)
	.into();

	// Filter out all successful (or already existing) results
	let errors: Vec<io::Error> = results
		.into_iter()
		.filter(Result::is_err)
		.map(|res| res.expect_err("somehow had a non-error error when checking app dir creation for errors"))
		.filter(|err| err.kind() != io::ErrorKind::AlreadyExists)
		.collect();

	if errors.is_empty() {
		Ok(())
	} else {
		Err(errors
			.into_iter()
			.map(|err| err.to_string())
			.collect::<Vec<_>>()
			.join(", "))
	}
}

/// Auto-discovers a Resonite path if the setting isn't configured
async fn autodiscover_resonite_path(app: &AppHandle) -> Result<(), anyhow::Error> {
	let path_configured = settings::get::<String>(app, "resonitePath")?.is_some();

	// If the path isn't already configured, try to find one automatically
	if !path_configured {
		info!("Resonite path not configured, running autodiscovery");

		// Run discovery
		let resonite_dir = async_runtime::spawn_blocking(|| discover::resonite(None))
			.await
			.context("Unable to spawn blocking task for Resonite path autodiscovery")??;

		// If discovery found a path, save it to the setting
		match resonite_dir {
			Some(resonite_dir) => {
				info!("Discovered Resonite path: {}", resonite_dir.display());
				settings::set(app, "resonitePath", &resonite_dir)?;

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
#[allow(clippy::needless_pass_by_value)]
fn build_error_window(app: &AppHandle, err: anyhow::Error) {
	let init_script = format!("globalThis.error = `{err:?}`;");
	WebviewWindowBuilder::new(app, "error", WebviewUrl::App("error.html".into()))
		.title("Resolute")
		.center()
		.resizable(false)
		.visible(false)
		.initialization_script(&init_script)
		.build()
		.expect("Error occurred while initializing and the error window couldn't be displayed");
	let _ = app
		.get_webview_window("main")
		.expect("unable to get main window")
		.close();
}

/// Builds a manifest config that takes the user-configured settings into account
pub(crate) fn build_manifest_config(app: &AppHandle) -> Result<manifest::Config, String> {
	// Build the base config
	let mut config = manifest::Config::default();
	config.cache_file_path = Some(
		app.path()
			.app_cache_dir()
			.map_err(|err| format!("Unable to locate cache directory: {err}"))?
			.join("resonite-mod-manifest.json"),
	);

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
	// Get the timeout from the settings store
	let connect_timeout: f32 = settings::get(app, "connectTimeout")?.unwrap_or(10f32);
	debug!("Building HTTP client, connectTimeout = {}s", connect_timeout);

	// Grab some details about the application
	let config = &app.config();
	let name = config
		.product_name
		.as_ref()
		.ok_or_else(|| anyhow!("Unable to get app product name"))?;
	let version = config
		.version
		.as_ref()
		.ok_or_else(|| anyhow!("Unable to get app version"))?;

	// Build the client
	reqwest::Client::builder()
		.connect_timeout(Duration::from_secs_f32(connect_timeout))
		.user_agent(format!("{}/{} ({})", name, version, env::consts::OS))
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
