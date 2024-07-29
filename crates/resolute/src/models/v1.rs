#![allow(clippy::exhaustive_structs, clippy::absolute_paths)]

use std::collections::HashMap;

#[cfg(feature = "db")]
use native_db::{native_db, ToKey};
#[cfg(feature = "db")]
use native_model::{native_model, Model};
use semver::Version;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{ModAuthor, ModDependencyMap};

/// First version of the [`super::ResoluteMod`] struct, kept around for database migration purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl From<ResoluteMod> for super::v2::ResoluteMod {
	fn from(value: ResoluteMod) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			category: value.category,
			authors: value.authors,
			source_location: value.source_location,
			website: value.website,
			tags: value.tags,
			flags: value.flags,
			platforms: value.platforms,
			versions: value.versions.into_iter().map(|(svr, ver)| (svr, ver.into())).collect(),
			active: value.installed_version.is_some(),
			installed_version: value.installed_version,
		}
	}
}

impl From<super::v2::ResoluteMod> for ResoluteMod {
	fn from(value: super::v2::ResoluteMod) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			category: value.category,
			authors: value.authors,
			source_location: value.source_location,
			website: value.website,
			tags: value.tags,
			flags: value.flags,
			platforms: value.platforms,
			versions: value.versions.into_iter().map(|(svr, ver)| (svr, ver.into())).collect(),
			installed_version: value.installed_version,
		}
	}
}

/// First version of the [`super::ModVersion`] struct, kept around for database migration purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
	pub semver: Version,
	pub artifacts: Vec<ModArtifact>,
	pub dependencies: ModDependencyMap,
	pub conflicts: ModDependencyMap,
	#[serde(rename = "releaseUrl")]
	pub release_url: Option<Url>,
}

impl From<ModVersion> for super::v2::ModVersion {
	fn from(value: ModVersion) -> Self {
		Self {
			semver: value.semver,
			artifacts: value.artifacts.into_iter().map(Into::into).collect(),
			dependencies: value.dependencies,
			conflicts: value.conflicts,
			release_url: value.release_url,
			changelog: None,
		}
	}
}

impl From<super::v2::ModVersion> for ModVersion {
	fn from(value: super::v2::ModVersion) -> Self {
		Self {
			semver: value.semver,
			artifacts: value.artifacts.into_iter().map(Into::into).collect(),
			dependencies: value.dependencies,
			conflicts: value.conflicts,
			release_url: value.release_url,
		}
	}
}

/// First version of the [`super::ModArtifact`] struct, kept around for database migration purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModArtifact {
	pub url: Url,
	pub sha256: String,
	pub filename: Option<String>,
	#[serde(rename = "installLocation")]
	pub install_location: Option<String>,
}

impl From<ModArtifact> for super::v2::ModArtifact {
	fn from(value: ModArtifact) -> Self {
		Self {
			url: value.url,
			sha256: value.sha256,
			filename: value.filename,
			install_location: value.install_location,
			override_filename: None,
		}
	}
}

impl From<super::v2::ModArtifact> for ModArtifact {
	fn from(value: super::v2::ModArtifact) -> Self {
		Self {
			url: value.url,
			sha256: value.sha256,
			filename: value.filename,
			install_location: value.install_location,
		}
	}
}
