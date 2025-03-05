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
		 * @type {?Array<'deprecated'|'plugin'|'final'>}
		 */
		this.flags = data.flags;

		/**
		 * Platforms the mod is supported on
		 * @type {Array<'android'|'headless'|'linux'|'linux-native'|'linux-wine'|'windows'|'other'>}
		 */
		this.platforms = data.platforms;

		/**
		 * Available versions mapped by semver string -> {@link ModVersion}
		 * @type {Object}
		 */
		this.versions = Object.fromEntries(
			Object.entries(data.versions)
				.map(([semver, version]) => [semver, new ModVersion(version)])
				.sort((a, b) => -semverCompare(a[0], b[0])),
		);

		/**
		 * Semver of the version that is currently installed
		 * @type {?ModVersion}
		 */
		this.installedVersion = data.installedVersion
			? this.versions[data.installedVersion]
			: null;

		/**
		 * Whether the mod is enabled
		 * @type {bool}
		 */
		this.active = data.active;
	}

	/**
	 * Whether this is an unrecognized mod
	 * @type {boolean}
	 */
	get isUnrecognized() {
		return this.id.startsWith('dev.gawdl3y.resolute.unrecognized');
	}

	/**
	 * Whether this mod has been deprecated (the {@link flags} property has 'deprecated' in it)
	 * @type {boolean}
	 */
	get isDeprecated() {
		return this.flags?.includes?.('deprecated') ?? false;
	}

	/**
	 * Latest available version
	 * @type {ModVersion}
	 */
	get latestVersion() {
		return Object.values(this.versions)[0];
	}

	/**
	 * Whether an update is available (the installed version is older than the latest version)
	 */
	get hasUpdate() {
		if (!this.installedVersion) return false;
		return semverLt(this.installedVersion.semver, this.latestVersion.semver);
	}

	/**
	 * A CSS class to use for the version text.
	 * text-success if installed and up-to-date, text-warning if there is an update available, nothing otherwise.
	 * @type {string}
	 */
	get versionTextClass() {
		if (!this.installedVersion) return '';
		if (this.isUnrecognized) return 'text-blue';
		return this.hasUpdate ? 'text-warning' : 'text-success';
	}

	/**
	 * A number for the version status, for the purposes of sorting.
	 * 0 for update available, 1 for installed, 2 for not installed
	 * @type {number}
	 */
	get sortableVersionStatus() {
		return this.hasUpdate ? 0 : this.installedVersion ? 1 : 2;
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
		 * @type {Object}
		 */
		this.dependencies = data.dependencies;

		/**
		 * Mod conflicts, in the form of a map of mod IDs -> conflicting semver range
		 * @type {Object}
		 */
		this.conflicts = data.conflicts;

		/**
		 * URL to a release page
		 * @type {?string}
		 */
		this.releaseUrl = data.releaseUrl;
	}

	/**
	 * Whether this version is for an unrecognized artifact
	 * @type {boolean}
	 */
	get isUnrecognized() {
		return this.semver === '0.0.0-unknown';
	}

	/**
	 * Text label for the version
	 * @type {string}
	 */
	get label() {
		return this.isUnrecognized ? 'Unknown' : this.semver;
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

	/**
	 * The inferred filename of the artifact (obtained from the end of the URL)
	 * @returns {string}
	 */
	get inferredFilename() {
		const url = new URL(this.url);
		const lastSlashIdx = url.pathname.lastIndexOf('/');
		return url.pathname.substring(lastSlashIdx + 1);
	}

	/**
	 * Gets the inferred install location of the artifact
	 * (always "/rml_mods" unless the given category is "Plugins", in which case it's "/Libraries")
	 * @returns {string}
	 */
	inferredInstallLocation(category) {
		if (category === 'Plugins') return '/Libraries';
		return '/rml_mods';
	}
}
