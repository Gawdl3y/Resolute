use std::{
	collections::HashMap,
	ffi::OsString,
	fmt::Display,
	path::{Path, PathBuf},
};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use url::Url;

#[cfg(feature = "db")]
use native_db::*;
#[cfg(feature = "db")]
use native_model::{native_model, Model};

use crate::manifest::{
	ManifestAuthors, ManifestData, ManifestEntryArtifact, ManifestEntryDependencies, ManifestEntryVersions,
};

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
