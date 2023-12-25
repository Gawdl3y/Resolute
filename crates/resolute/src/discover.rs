use std::path::PathBuf;

use log::debug;
use steamlocate::SteamDir;

use crate::Result;

pub const RESONITE_APP: u32 = 2_519_830;

/// Searches for a potenial Resonite game directory
pub fn discover_resonite(steam: Option<SteamDir>) -> Result<Option<PathBuf>> {
	// Find a Steam installation if one isn't provided
	let steam = match steam {
		Some(steam) => steam,
		None => {
			let steam = SteamDir::locate()?;
			debug!("Steam installation located at {}", steam.path().display());
			steam
		}
	};

	// Check the Steam installation for Resonite
	let resonite_details = steam.find_app(RESONITE_APP)?;
	match resonite_details {
		Some((resonite, library)) => {
			let resonite_dir = library.resolve_app_dir(&resonite);
			debug!("Resonite installation located at {}", resonite_dir.display());
			Ok(Some(resonite_dir))
		}
		None => Ok(None),
	}
}
