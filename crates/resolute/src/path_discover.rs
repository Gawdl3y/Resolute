use std::path::PathBuf;

use log::debug;
use tokio::fs;

use crate::{Error, Result};

/// Searches for a potenial Resonite game directory
pub async fn discover_resonite() -> Result<Option<PathBuf>> {
	let steam_path = discover_steam().await?;
	match steam_path {
		Some(steam_path) => {
			// Verify there is a Resonite game directory in the Steam directory
			let resonite_path = steam_path.join("steamapps/common/Resonite");
			if fs::try_exists(&resonite_path).await? {
				debug!("Resonite found at {}", resonite_path.display());
				Ok(Some(resonite_path))
			} else {
				Ok(None)
			}
		}

		None => Ok(None),
	}
}

/// Searches for a potential Steam installation directory by reading the Windows registry or defaulting to the standard path
#[cfg(target_os = "windows")]
pub async fn discover_steam() -> Result<Option<PathBuf>> {
	use winreg::{enums::HKEY_CURRENT_USER, RegKey};

	// Locate a Steam installation from the registry or the default installation path
	let steam_path = PathBuf::from({
		let hklm = RegKey::predef(HKEY_CURRENT_USER);
		hklm.open_subkey("Software\\Valve\\Steam")
			.and_then(|key| {
				debug!("Opened Steam registry key, reading SteamPath value from it");
				key.get_value("SteamPath")
			})
			.or_else(|err| {
				debug!(
					"Error reading SteamPath value from registry (falling back to default path): {}",
					err
				);
				Ok::<String, Error>("%ProgramFiles(x86)%/Steam".to_owned())
			})
			.unwrap()
	});

	// Confirm the existence of the Steam directory
	if fs::try_exists(&steam_path).await? {
		debug!("Steam found at {}, canonicalizing path", steam_path.display());
		Ok(Some(fs::canonicalize(steam_path).await?))
	} else {
		Ok(None)
	}
}

#[cfg(target_os = "linux")]
pub async fn discover_steam() -> Result<Option<PathBuf>> {
	Ok(None)
}

#[cfg(target_os = "macos")]
pub async fn discover_steam() -> Result<Option<PathBuf>> {
	Err(Error::UnsupportedPlatform("macos".to_owned()))
}
