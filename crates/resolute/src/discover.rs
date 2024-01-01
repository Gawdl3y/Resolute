use std::{
	collections::HashMap,
	fs, io,
	path::{Path, PathBuf},
};

use log::{debug, error, trace};
use sha2::{Digest, Sha256};
use steamlocate::SteamDir;

use crate::{
	manager::ArtifactPaths,
	mods::{ResoluteMod, ResoluteModMap},
	Result,
};

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

/// Searches for any installed mods in a Resonite directory
pub fn discover_mods(base_path: impl AsRef<Path>, mods: ResoluteModMap) -> Result<ResoluteModMap> {
	let mut discovered = ResoluteModMap::new();
	let mut checksums: HashMap<PathBuf, Option<String>> = HashMap::new();
	let base_path = base_path.as_ref().to_path_buf();

	'mods: for (id, rmod) in mods {
		'versions: for (semver, version) in &rmod.versions {
			trace!("Scanning for artifacts from mod {} v{}", rmod, semver);

			'_artifacts: for artifact in &version.artifacts {
				trace!("Checking for artifact {} from mod {} v{}", artifact, rmod, semver);

				// Build the path that the artifact would use
				let path = match ArtifactPaths::try_new(artifact, &base_path) {
					Ok(paths) => base_path.join(paths.final_dest),
					Err(_err) => continue 'versions,
				};

				// Check the checksum cache for the file
				match checksums.get(&path) {
					// Checksum has already been calculated - just check it against the artifact's hash and
					// move on if it doesn't match
					Some(Some(checksum)) => {
						if *checksum != artifact.sha256.to_lowercase() {
							trace!(
								"Artifact {} checksum mismatch (expected = {}, actual(cached) = {})",
								artifact,
								artifact.sha256,
								checksum
							);
							continue 'versions;
						}
					}

					// File has been encountered but doesn't exist - move on to the next version
					Some(None) => {
						trace!("Artifact {} file doesn't exist (cached)", artifact);
						continue 'versions;
					}

					// File hasn't yet been encountered
					None => {}
				};

				// Open the file - if we're unable to, add that fact to the checksum cache
				let mut file = match fs::File::open(&path) {
					Ok(file) => file,
					Err(err) => {
						trace!("Artifact file {} can't be opened: {}", artifact, err);
						checksums.insert(path, None);
						continue 'versions;
					}
				};

				// Calculate the file's checksum
				let mut hasher = Sha256::new();
				let hash = match io::copy(&mut file, &mut hasher) {
					Ok(_bytes) => format!("{:x}", hasher.finalize()),
					Err(err) => {
						error!("Error hashing artifact file {}: {}", artifact, err);
						continue 'versions;
					}
				};
				checksums.insert(path, Some(hash.clone()));

				// If the hash doesn't match, move on to the next version
				if hash != artifact.sha256.to_lowercase() {
					trace!(
						"Artifact {} checksum mismatch (expected = {}, actual = {})",
						artifact,
						artifact.sha256,
						hash
					);
					continue 'versions;
				}
			}

			// A matched version has been found
			debug!("Discovered installed mod {} v{}", rmod, semver);
			let rmod = ResoluteMod {
				installed_version: Some(semver.clone()),
				..rmod.clone()
			};
			discovered.insert(id.clone(), rmod);
			continue 'mods;
		}
	}

	Ok(discovered)
}
