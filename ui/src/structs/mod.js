import { compare as semverCompare, lt as semverLt } from 'semver';

/**
 * Container for all data about a Resonite mod
 */
export class ResoluteMod {
	constructor(data) {
		/**
		 * Full unique identifier
		 * @type {string}
		 */
		this.id = data.id;

		/**
		 * Short name
		 * @type {string}
		 */
		this.name = data.name;

		/**
		 * Summary of the mod's purpose
		 */
		this.description = data.description;

		/**
		 * Category the mod belongs to
		 * @type {string}
		 */
		this.category = data.category;

		/**
		 * Creators/contributors of the mod, with the first one being the primary author
		 * @type {ModAuthor[]}
		 */
		this.authors = data.authors.map((author) => new ModAuthor(author));

		/**
		 * URL to the source code for the mod
		 * @type {?string}
		 */
		this.sourceLocation = data.sourceLocation;

		/**
		 * URL to the homepage for the mod
		 * @type {?string}
		 */
		this.website = data.website;

		/**
		 * List of searchable tags
		 * @type {string[]}
		 */
		this.tags = data.tags;

		/**
		 * Meta flags
		 * @type {Array<'deprecated'|'plugin'|'final'>}
		 */
		this.flags = data.flags;

		/**
		 * Platforms the mod is supported on
		 * @type {Array<'android'|'headless'|'linux'|'linux-native'|'linux-wine'|'windows'|'other'>}
		 */
		this.platforms = data.platforms;

		/**
		 * Available versions
		 * @type {Map<string, ModVersion>}
		 */
		this.versions = new Map(
			Object.entries(data.versions)
				.map(([semver, version]) => [semver, new ModVersion(version)])
				.sort((a, b) => -semverCompare(a[0], b[0])),
		);

		/**
		 * Semver of the version that is currently installed
		 * @type {?ModVersion}
		 */
		this.installedVersion = data.installedVersion
			? this.versions.get(data.installedVersion)
			: null;
	}

	/**
	 * Latest available version
	 * @type {ModVersion}
	 */
	get latestVersion() {
		return this.versions.values().next().value;
	}

	/**
	 * Whether an update is available (the installed version is older than the latest version)
	 */
	get hasUpdate() {
		if (!this.installedVersion) return false;
		return semverLt(this.installedVersion.semver, this.latestVersion.semver);
	}

	/**
	 * A number for the version status, for the purposes of sorting.
	 * 0 for update available, 1 for installed, 2 for not installed
	 * @type {number}
	 */
	get sortableVersionStatus() {
		return this.hasUpdate ? 0 : this.installedVersion ? 1 : 2;
	}

	/**
	 * A CSS class to use for the version text.
	 * text-success if installed and up-to-date, text-warning if there is an update available, nothing otherwise.
	 * @type {string}
	 */
	get versionTextClass() {
		if (!this.installedVersion) return '';
		return this.hasUpdate ? 'text-warning' : 'text-success';
	}

	toJSON() {
		return {
			...this,
			installedVersion: this.installedVersion?.semver,
		};
	}
}

/**
 * Contributor to a {@link ResoluteMod}
 */
export class ModAuthor {
	constructor(data) {
		/**
		 * Name/username
		 * @type {string}
		 */
		this.name = data.name;

		/**
		 * URL to the author's homepage
		 * @type {?string}
		 */
		this.url = data.url;

		/**
		 * URL to an avatar/icon
		 * @type {?string}
		 */
		this.icon = data.icon;

		/**
		 * URL to a support page
		 * @type {?string}
		 */
		this.support = data.support;
	}
}

/**
 * Available version for a {@link ResoluteMod}
 */
export class ModVersion {
	constructor(data) {
		/**
		 * Semver version string
		 * @type {string}
		 */
		this.semver = data.semver;

		/**
		 * Files to install
		 * @type {ModArtifact[]}
		 */
		this.artifacts = data.artifacts.map(
			(artifact) => new ModArtifact(artifact),
		);

		/**
		 * Required mods, in the form of a map of mod IDs -> required semver range
		 * @type {Map<string, string>}
		 */
		this.dependencies = new Map(Object.entries(data.dependencies));

		/**
		 * Mod conflicts, in the form of a map of mod IDs -> conflicting semver range
		 * @type {Map<string, string>}
		 */
		this.conflicts = new Map(Object.entries(data.conflicts));

		/**
		 * URL to a release page
		 * @type {?string}
		 */
		this.releaseUrl = data.releaseUrl;
	}
}

/**
 * File to install for a {@link ModVersion}
 */
export class ModArtifact {
	constructor(data) {
		/**
		 * URL to the file to download
		 * @type {string}
		 */
		this.url = data.url;

		/**
		 * SHA-256 checksum of the file
		 * @type {string}
		 */
		this.sha256 = data.sha256;

		/**
		 * Filename to name the downloaded file as
		 * @type {?string}
		 */
		this.filename = data.filename;

		/**
		 * Location to download the file to
		 * @type {?string}
		 */
		this.installLocation = data.installLocation;
	}
}
