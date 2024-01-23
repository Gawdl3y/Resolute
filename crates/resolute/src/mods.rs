use std::{
	collections::HashMap,
	ffi::OsString,
	fmt::Display,
	path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use path_clean::PathClean;
use semver::{BuildMetadata, Prerelease, Version, VersionReq};
use serde::{Deserialize, Serialize};
use url::Url;

#[cfg(feature = "db")]
use native_db::*;
#[cfg(feature = "db")]
use native_model::{native_model, Model};

use crate::{
	manifest::{
		ManifestAuthors, ManifestData, ManifestEntryArtifact, ManifestEntryDependencies, ManifestEntryVersions,
	},
	Error,
};

/// Group string for unrecognized mods
pub const UNRECOGNIZED_GROUP: &str = "dev.gawdl3y.resolute.unrecognized";

/// Semver representing an unknown version
pub static UNRECOGNIZED_SEMVER: Lazy<Version> = Lazy::new(|| Version {
	major: 0,
	minor: 0,
	patch: 0,
	pre: Prerelease::new("unknown").expect("unable to create prerelease struct for unrecognized semver"),
	build: BuildMetadata::default(),
});

/// Base URL for an unrecognized artifact
pub static UNRECOGNIZED_ARTIFACT_BASE_URL: Lazy<Url> = Lazy::new(|| {
	Url::parse("resolute://unrecognized/artifact").expect("unable to parse unrecognized artifact base url")
});

/// Builds a mod map from the given raw manifest data
pub fn load_manifest(manifest: ManifestData) -> ResoluteModMap {
	manifest
		.objects
		.into_values()
		.flat_map(|object| {
			// Build the list of authors for the group
			let group_authors = build_mod_authors(object.authors);

			// Build the list of mods
			object.entries.into_iter().map(move |(id, entry)| {
				// Combine the group authors and mod's additional authors
				let mut authors = group_authors.clone();
				if let Some(additional_authors) = entry.additional_authors {
					authors.append(&mut build_mod_authors(additional_authors));
				}

				ResoluteMod {
					id,
					authors,
					versions: build_mod_versions_map(entry.versions, &entry.category),
					name: entry.name,
					description: entry.description,
					category: entry.category,
					source_location: entry.source_location,
					website: entry.website,
					tags: entry.tags,
					flags: entry.flags,
					platforms: entry.platforms,
					installed_version: None,
				}
			})
		})
		.map(|rmod| (rmod.id.clone(), rmod))
		.collect()
}

/// Build an authors list from manifest data
fn build_mod_authors(authors: ManifestAuthors) -> Vec<ModAuthor> {
	authors
		.into_iter()
		.map(|(name, author)| ModAuthor {
			name,
			url: author.url,
			icon: author.icon,
			support: author.support,
		})
		.collect()
}

/// Build a versions map from manifest data
fn build_mod_versions_map(versions: ManifestEntryVersions, category: &str) -> HashMap<Version, ModVersion> {
	versions
		.into_iter()
		.map(|(semver, version)| ModVersion {
			semver,
			dependencies: build_mod_version_dependencies(version.dependencies),
			conflicts: build_mod_version_dependencies(version.conflicts),
			artifacts: build_mod_version_artifacts(version.artifacts, category),
			release_url: version.release_url,
		})
		.map(|version| (version.semver.clone(), version))
		.collect()
}

/// Build a dependencies map from manifest data for a mod version
fn build_mod_version_dependencies(dependencies: Option<ManifestEntryDependencies>) -> HashMap<String, VersionReq> {
	if let Some(depends) = dependencies {
		depends
			.into_iter()
			.map(|(depend_id, depend)| (depend_id, depend.version))
			.collect()
	} else {
		HashMap::new()
	}
}

/// Build an artifacts map from manifest data for a mod version
fn build_mod_version_artifacts(artifacts: Vec<ManifestEntryArtifact>, category: &str) -> Vec<ModArtifact> {
	artifacts
		.into_iter()
		.map(|artifact| ModArtifact::from_manifest_and_category(artifact, category))
		.collect()
}

/// ResoniteMods mapped by their ID
pub type ResoluteModMap = HashMap<String, ResoluteMod>;

/// A single Resonite mod with all information relevant to it
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "db", native_model(id = 1, version = 1))]
#[cfg_attr(feature = "db", native_db)]
pub struct ResoluteMod {
	// The primary_key and secondary_key macros don't work with cfg_attr for whatever reason
	#[cfg(feature = "db")]
	#[primary_key]
	pub id: String,
	#[cfg(not(feature = "db"))]
	pub id: String,

	// The primary_key and secondary_key macros don't work with cfg_attr for whatever reason
	#[cfg(feature = "db")]
	#[secondary_key]
	pub name: String,
	#[cfg(not(feature = "db"))]
	pub name: String,

	pub description: String,
	pub category: String,
	pub authors: Vec<ModAuthor>,
	#[serde(rename = "sourceLocation")]
	pub source_location: Option<Url>,
	pub website: Option<Url>,
	pub tags: Option<Vec<String>>,
	pub flags: Option<Vec<String>>,
	pub platforms: Option<Vec<String>>,
	pub versions: HashMap<Version, ModVersion>,
	#[serde(rename = "installedVersion")]
	pub installed_version: Option<Version>,
}

impl ResoluteMod {
	/// Gets the latest version available for the mod
	pub fn latest_version(&self) -> Option<&ModVersion> {
		self.versions.values().max_by(|a, b| a.semver.cmp(&b.semver))
	}

	/// Checks whether there is a newer version of the mod available than the installed version.
	/// If the mod isn't installed, None is returned.
	pub fn has_update(&self) -> Option<bool> {
		match &self.installed_version {
			Some(installed_version) => Some(self.latest_version()?.semver.gt(installed_version)),
			None => None,
		}
	}

	/// Checks whether this mod is unrecognized (ID begins with [UNRECOGNIZED_GROUP])
	pub fn is_unrecognized(&self) -> bool {
		self.id.starts_with(UNRECOGNIZED_GROUP)
	}

	/// Creates a new unrecognized mod from details about an encountered artifact file
	pub fn new_unrecognized(
		artifact_filename: impl AsRef<str>,
		artifact_install_location: impl AsRef<str>,
		artifact_sha256: impl AsRef<str>,
	) -> Self {
		let artifact_filename = artifact_filename.as_ref();
		let artifact_path = PathBuf::from(artifact_filename);
		let artifact_stem = artifact_path
			.file_stem()
			.map(|stem| {
				stem.to_str()
					.expect("unable to convert artifact filename stem to string")
			})
			.unwrap_or(artifact_filename);

		let mut versions = HashMap::new();
		let version = ModVersion::new_unrecognized(artifact_filename, &artifact_install_location, artifact_sha256);
		let semver = version.semver.clone();
		versions.insert(semver.clone(), version);

		Self {
			id: format!("{}.{}", UNRECOGNIZED_GROUP, artifact_stem.replace(' ', "-")),
			name: artifact_stem.to_owned(),
			description: format!("Unrecognized mod discovered in {}", artifact_install_location.as_ref()),
			category: "Unrecognized".to_owned(),
			authors: vec![ModAuthor::unknown()],
			source_location: None,
			website: None,
			tags: Some(vec!["unrecognized".to_owned()]),
			flags: None,
			platforms: None,
			versions,
			installed_version: Some(semver),
		}
	}
}

impl Display for ResoluteMod {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} ({})", self.name, self.id)
	}
}

/// Details for an author of a mod
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModAuthor {
	pub name: String,
	pub url: Option<Url>,
	pub icon: Option<Url>,
	pub support: Option<Url>,
}

impl ModAuthor {
	/// Creates a new unknown author
	pub fn unknown() -> Self {
		Self {
			name: "Unknown".to_owned(),
			url: None,
			icon: None,
			support: None,
		}
	}
}

impl Display for ModAuthor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}

/// Details for a released version of a mod
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModVersion {
	pub semver: Version,
	pub artifacts: Vec<ModArtifact>,
	pub dependencies: ModDependencyMap,
	pub conflicts: ModDependencyMap,
	#[serde(rename = "releaseUrl")]
	pub release_url: Option<Url>,
}

impl ModVersion {
	/// Checks whether this version is unrecognized (semver equals [UNRECOGNIZED_SEMVER])
	pub fn is_unrecognized(&self) -> bool {
		self.semver.eq(&UNRECOGNIZED_SEMVER)
	}

	/// Creates a new unrecognized version from details about a single encountered artifact file
	pub fn new_unrecognized(
		artifact_filename: impl AsRef<str>,
		artifact_install_location: impl AsRef<str>,
		artifact_sha256: impl AsRef<str>,
	) -> Self {
		let artifacts = vec![ModArtifact::new_unrecognized(
			artifact_filename,
			artifact_install_location,
			artifact_sha256,
		)];

		Self {
			semver: UNRECOGNIZED_SEMVER.clone(),
			artifacts,
			dependencies: ModDependencyMap::new(),
			conflicts: ModDependencyMap::new(),
			release_url: None,
		}
	}

	/// Creates a new unrecognized version with a list of artifacts
	pub fn new_unrecognized_with_artifacts(artifacts: Vec<ModArtifact>) -> Self {
		Self {
			semver: UNRECOGNIZED_SEMVER.clone(),
			artifacts,
			dependencies: ModDependencyMap::new(),
			conflicts: ModDependencyMap::new(),
			release_url: None,
		}
	}
}

impl Display for ModVersion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.semver)
	}
}

/// Details for a release artifact
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModArtifact {
	pub url: Url,
	pub sha256: String,
	pub filename: Option<String>,
	#[serde(rename = "installLocation")]
	pub install_location: Option<String>,
}

impl ModArtifact {
	/// Gets the filename from the end of the artifact's URL
	pub fn infer_filename(&self) -> Option<OsString> {
		let path = Path::new(self.url.path());
		path.file_name().map(|filename| filename.to_owned())
	}

	/// Gets the default install location for the artifact, influenced by the category of the mod if available
	pub fn infer_install_location(&self, category: Option<impl AsRef<str>>) -> PathBuf {
		match category {
			Some(category) => match category.as_ref() {
				"Plugins" => PathBuf::from("Libraries"),
				_ => PathBuf::from("rml_mods"),
			},
			None => PathBuf::from("rml_mods"),
		}
	}

	/// Gets the filename or inferred filename. Panics if neither can be obtained.
	pub fn usable_filename(&self) -> OsString {
		self.filename
			.as_ref()
			.map(|filename| OsString::from(&filename))
			.or_else(|| self.infer_filename())
			.expect("unable to get filename of artifact")
	}

	/// Checks whether this artifact is unrecognized (URL begins with [UNRECOGNIZED_ARTIFACT_BASE_URL])
	pub fn is_unrecognized(&self) -> bool {
		self.url.as_str().starts_with(UNRECOGNIZED_ARTIFACT_BASE_URL.as_str())
	}

	/// Gets the full final destination path for the artifact within a base path.
	/// Fails if the final destination is outside of the base path or if there are any issues building the path.
	pub fn dest_within(&self, base_path: impl AsRef<Path>) -> crate::Result<PathBuf> {
		let base_path = base_path.as_ref();

		// Add the artifact's install location to the path
		let mut dest = base_path.join(match &self.install_location {
			Some(install_location) => {
				let path = Path::new(install_location);
				path.strip_prefix("/").or::<Error>(Ok(path))?
			}
			None => Path::new("rml_mods"),
		});

		// Add the artifact's filename to the path
		let filename = match &self.filename {
			Some(filename) => OsString::from(filename),
			None => self
				.infer_filename()
				.ok_or_else(|| Error::Path(format!("unable to infer filename from url: {}", self.url)))?
				.to_owned(),
		};
		dest.push(&filename);

		// Ensure the final path is inside the base path
		let final_dest = dest.clean();
		if !final_dest.starts_with(base_path) {
			return Err(Error::Path(
				"artifact's final destination is not a subdirectory of the base destination".to_owned(),
			));
		}

		Ok(final_dest)
	}

	/// Creates a new unrecognized artifact from details about an encountered artifact file
	pub fn new_unrecognized(
		filename: impl AsRef<str>,
		install_location: impl AsRef<str>,
		sha256: impl AsRef<str>,
	) -> Self {
		let filename = filename.as_ref();
		let install_location = install_location.as_ref();
		let sha256 = sha256.as_ref();

		let mut url = UNRECOGNIZED_ARTIFACT_BASE_URL.clone();
		url.path_segments_mut()
			.expect("unable to get mutable path segments of unrecognized artifact base url")
			.push(install_location)
			.push(filename);

		let install_location = if !install_location.starts_with('/') {
			let mut install_location = install_location.to_owned();
			install_location.insert(0, '/');
			install_location
		} else {
			install_location.to_owned()
		};

		ModArtifact {
			url,
			sha256: sha256.to_owned(),
			filename: Some(filename.to_owned()),
			install_location: Some(install_location),
		}
	}

	/// Gets the temporary destination path for an artifact from its final destination path.
	/// Fails if there is no filename in the input path.
	pub fn tmp_dest(dest: impl AsRef<Path>) -> crate::Result<PathBuf> {
		let dest = dest.as_ref();
		let mut filename = dest
			.file_name()
			.ok_or_else(|| {
				Error::Path(format!(
					"unable to build temporary destination for final destination ({})",
					dest.display()
				))
			})?
			.to_owned();
		filename.push(".new");
		Ok(dest.with_file_name(filename))
	}

	/// Gets the old (existing, being replaced) destination path for an artifact from its final destination path.
	/// Fails if there is no filename in the input path.
	pub fn old_dest(dest: impl AsRef<Path>) -> crate::Result<PathBuf> {
		let dest = dest.as_ref();
		let mut filename = dest
			.file_name()
			.ok_or_else(|| {
				Error::Path(format!(
					"unable to build old destination for final destination ({})",
					dest.display()
				))
			})?
			.to_owned();
		filename.push(".old");
		Ok(dest.with_file_name(filename))
	}
}

impl Display for ModArtifact {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let name = self
			.filename
			.clone()
			.or_else(|| {
				Path::new(self.url.path())
					.file_name()
					.and_then(|name| name.to_str())
					.map(|name| name.to_owned())
			})
			.or_else(|| Some(self.url.to_string()))
			.unwrap();

		write!(f, "{}", name)
	}
}

impl ModArtifact {
	fn from_manifest_and_category(value: ManifestEntryArtifact, category: &str) -> Self {
		Self {
			url: value.url,
			sha256: value.sha256,
			filename: value.filename,
			install_location: value.install_location.or_else(|| match category {
				"Plugins" => Some("/Libraries".to_owned()),
				_ => None,
			}),
		}
	}
}

impl From<ManifestEntryArtifact> for ModArtifact {
	fn from(value: ManifestEntryArtifact) -> Self {
		Self {
			url: value.url,
			sha256: value.sha256,
			filename: value.filename,
			install_location: value.install_location,
		}
	}
}

/// Map of mod IDs to semver ranges
pub type ModDependencyMap = HashMap<String, VersionReq>;
