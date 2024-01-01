use std::{
	ffi::OsString,
	path::{Path, PathBuf},
};

use path_clean::PathClean;

use crate::{mods::ModArtifact, Error, Result};

/// Contains full paths that an artifact may live in at various stages of installation
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ArtifactPaths {
	/// Full path for an artifact file that has been installed
	pub final_dest: PathBuf,
	/// Full path for an artifact file that has just been downloaded
	pub tmp_dest: PathBuf,
	/// Full path for an artifact file that already existed and has been renamed
	pub old_dest: PathBuf,
}

impl ArtifactPaths {
	/// Builds a set of artifact destination paths for a given artifact and base destination path.
	/// Fails if there's any issue building the paths or if the artifact's destination ends up outside of the base path.
	pub fn try_new(artifact: &ModArtifact, base_dest: impl AsRef<Path>) -> Result<Self> {
		let base_dest = base_dest.as_ref();

		// Add the artifact's install location to the path
		let mut dest = base_dest.join(match &artifact.install_location {
			Some(install_location) => {
				let path = Path::new(install_location);
				path.strip_prefix("/").or::<Error>(Ok(path))?
			}
			None => Path::new("rml_mods"),
		});

		// Add the artifact's filename to the path
		let filename = match &artifact.filename {
			Some(filename) => OsString::from(filename),
			None => Path::new(artifact.url.path())
				.file_name()
				.ok_or_else(|| Error::Path(format!("unable to extract file name from url: {}", artifact.url)))?
				.to_owned(),
		};
		dest.push(&filename);

		// Ensure the final path is inside the base path
		let final_dest = dest.clean();
		if !final_dest.starts_with(base_dest) {
			return Err(Error::Path(
				"artifact's final destination is not a subdirectory of the base destination".to_owned(),
			));
		}

		// Build the temporary and old filenames
		let mut tmp_filename = filename.clone();
		tmp_filename.push(".new");
		let mut old_filename = filename;
		old_filename.push(".old");

		Ok(Self {
			tmp_dest: final_dest.with_file_name(tmp_filename),
			old_dest: final_dest.with_file_name(old_filename),
			final_dest,
		})
	}
}

impl Eq for ArtifactPaths {}
