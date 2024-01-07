use std::{
	collections::HashMap,
	ffi::OsString,
	fs, io,
	path::{Path, PathBuf},
};

use log::{debug, error, trace};
use sha2::{Digest, Sha256};
use steamlocate::SteamDir;

use crate::{
	manager::ArtifactPaths,
	mods::{ModVersion, ResoluteMod, ResoluteModMap},
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

/// Searches for any installed mods in a Resonite directory by checksum then filename, including unrecognized mods
pub fn discover_mods(base_path: impl AsRef<Path>, mods: ResoluteModMap) -> Result<ResoluteModMap> {
	let mut discovered = discover_mods_by_checksum(&base_path, &mods)?;
	discovered.extend(discover_mods_by_filename(&base_path, &mods, Some(&discovered))?);
	Ok(discovered)
}

/// Searches for any installed mods in a Resonite directory using artifact checksums
pub fn discover_mods_by_checksum(base_path: impl AsRef<Path>, mods: &ResoluteModMap) -> Result<ResoluteModMap> {
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

/// Searches for any installed mods in a Resonite directory using artifact filenames
pub fn discover_mods_by_filename(
	base_path: impl AsRef<Path>,
	mods: &ResoluteModMap,
	already_discovered: Option<&ResoluteModMap>,
) -> Result<ResoluteModMap> {
	let mut discovered = ResoluteModMap::new();
	let base_path = base_path.as_ref();
	let search_dirs = vec!["rml_mods", "rml_libs"];

	for dirname in search_dirs {
		let path = base_path.join(dirname);
		trace!("Scanning for artifact files in {}", path.display());

		// Read the files in the path
		let files = match fs::read_dir(&path) {
			Ok(files) => files,
			Err(err) => {
				debug!("Artifact search path ({}) cannot be read: {}", path.display(), err);
				continue;
			}
		};

		for artifact_file in files {
			// Get the file details
			let artifact_file = match artifact_file {
				Ok(file) => file,
				Err(err) => {
					debug!("Artifact file cannot be listed: {}", err);
					continue;
				}
			};

			let artifact_path = artifact_file.path();
			debug!("Looking for mods for artifact file {}", artifact_path.display());

			// If the file is present in the already-discovered mods, skip it
			if let Some(already_discovered) = already_discovered {
				let exists = already_discovered.values().any(|rmod| {
					let installed_version = rmod
						.installed_version
						.as_ref()
						.expect("unable to get installed version of discovered mod");

					rmod.versions
						.get(installed_version)
						.expect("unable to get installed version from discovered mod versions map")
						.artifacts
						.iter()
						.any(|artifact| {
							artifact
								.filename
								.as_ref()
								.map(|filename| OsString::from(&filename))
								.or_else(|| artifact.infer_filename())
								.expect("unable to get filename of artifact")
								== artifact_file.file_name()
						})
				});

				if exists {
					debug!(
						"Artifact file {} is part of an already-discovered mod",
						artifact_path.display()
					);
					continue;
				}
			}

			// Get the first known mod that has this filename in its artifacts
			let existing_rmod = mods.values().find(|rmod| {
				rmod.versions.values().any(|version| {
					version.artifacts.iter().any(|artifact| {
						artifact
							.filename
							.as_ref()
							.map(|filename| OsString::from(&filename))
							.or_else(|| artifact.infer_filename())
							.expect("unable to get filename of artifact")
							== artifact_file.file_name()
					})
				})
			});

			// Calculate the checksum of the file
			let sha256 = {
				// Open the file
				let mut file = match fs::File::open(&artifact_path) {
					Ok(file) => file,
					Err(err) => {
						error!("Artifact file {} can't be opened: {}", artifact_path.display(), err);
						continue;
					}
				};

				// Calculate the checksum
				let mut hasher = Sha256::new();
				match io::copy(&mut file, &mut hasher) {
					Ok(_bytes) => format!("{:x}", hasher.finalize()),
					Err(err) => {
						error!("Error hashing artifact file {}: {}", artifact_path.display(), err);
						continue;
					}
				}
			};

			// Get a str representation of the filename
			let filename = artifact_file.file_name();
			let filename = match filename.to_str() {
				Some(filename) => filename,
				None => {
					error!(
						"Error converting artifact file name ({:?}) to string",
						artifact_file.file_name()
					);
					continue;
				}
			};

			// Create an unrecognized mod
			let rmod = match existing_rmod {
				Some(rmod) => {
					let version = ModVersion::new_unrecognized(filename, dirname, sha256);
					let semver = version.semver.clone();

					let mut rmod = rmod.clone();
					rmod.versions.insert(semver.clone(), version);
					rmod.installed_version = Some(semver.clone());

					debug!("Created unrecognized version {} for existing mod {}", semver, rmod);
					rmod
				}

				None => {
					let rmod = ResoluteMod::new_unrecognized(filename, dirname, sha256);
					debug!("Created unrecognized mod {}", rmod);
					rmod
				}
			};

			discovered.insert(rmod.id.clone(), rmod);
		}
	}

	Ok(discovered)
}
