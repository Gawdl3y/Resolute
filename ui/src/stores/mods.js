import { ref, reactive } from 'vue';
import { defineStore } from 'pinia';
import { lt as semverLt } from 'semver';
import { invoke } from '@tauri-apps/api/core';
import { info, error } from '@tauri-apps/plugin-log';

import useNotifications from '../composables/notifications';
// eslint-disable-next-line no-unused-vars
import { ResoluteMod, ModVersion } from '../structs/mod';

export const useModStore = defineStore('mods', () => {
	const mods = ref(null);
	const loading = ref(false);
	const loadingInstalled = ref(false);
	const discovering = ref(false);
	const hasLoaded = ref(false);
	const hasLoadedInstalled = ref(false);
	const operations = reactive({});
	const notify = useNotifications();

	/**
	 * Retrieves mod data from the backend
	 * @param {boolean} [bypassCache=false] Whether to bypass the manifest cache when possible
	 * @param {boolean} [alert=true] Whether to alert the user for failures
	 * @returns {Object} {@link ResoluteMod}s mapped by their ID
	 */
	async function load(bypassCache = false, alert = true) {
		// Ensure we're not already loading the mods elsewhere
		if (loading.value) throw new Error('Already loading mods.');
		loading.value = true;

		try {
			// Load the mods from the backend
			await info(`Requesting mod load, bypassCache = ${bypassCache}`);
			const { mods: newMods, removed } = await invoke('load_all_mods', {
				bypassCache,
			});
			for (const id of Object.keys(newMods)) {
				newMods[id] = new ResoluteMod(newMods[id]);
			}

			// Ditch any mods that were specifically removed
			if (mods.value && removed) {
				for (const id of Object.keys(removed)) {
					if (mods.value[id]) delete mods.value[id];
				}
			}

			// Mark the mods as loaded
			hasLoaded.value = true;
			console.debug('Mods loaded', newMods);
			info(`${Object.keys(newMods).length} mods loaded`);

			this.$patch({ mods: newMods });
			return newMods;
		} catch (err) {
			error(`Error loading mods: ${err}`);

			// Alert the user to the failure
			if (alert) {
				notify.error('Error loading mods', `Error loading mod list:\n${err}`);
			}

			throw err;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Retrieves installed mod data from the backend
	 * @returns {Object} {@link ResoluteMod}s mapped by their ID
	 */
	async function loadInstalled() {
		// Ensure we're not already loading the installed mods elsewhere
		if (loadingInstalled.value) {
			throw new Error('Already loading installed mods.');
		}
		loadingInstalled.value = true;

		try {
			// Load the installed mods from the backend
			await info('Requesting installed mod load');
			const { mods: newMods, removed } = await invoke('load_installed_mods');
			for (const id of Object.keys(newMods)) {
				newMods[id] = new ResoluteMod(newMods[id]);
			}

			// Ditch any mods that were specifically removed
			if (mods.value && removed) {
				for (const id of Object.keys(removed)) {
					if (mods.value[id]) delete mods.value[id];
				}
			}

			// Mark the installed mods as loaded
			hasLoadedInstalled.value = true;
			console.debug('Installed mods loaded', newMods);
			info(`${Object.keys(newMods).length} installed mods loaded`);

			this.$patch({ mods: newMods });
			return newMods;
		} catch (err) {
			// Alert the user to the failure
			error(`Error loading installed mods: ${err}`);
			notify.error(
				'Error loading mods',
				`Error loading installed mod list:\n${err}`,
			);
			throw err;
		} finally {
			loadingInstalled.value = false;
		}
	}

	/**
	 * Requests the installation of a mod from the backend and displays an alert when a result is received
	 * @param {ResoluteMod|string} mod
	 * @param {ModVersion|string} [version] Version to install (defaults to latest available)
	 */
	async function install(mod, version) {
		mod = typeof mod === 'string' ? mods.value[mod] : mod;

		// Determine the version to request the install of
		if (!version) version = mod.latestVersion;
		else if (typeof version === 'string') version = mod.versions[version];

		try {
			// Add an operation for the mod being installed and request the installation from the backend
			operations[mod.id] = 'install';
			await info(
				`Requesting installation of mod ${mod.name} v${version.semver}`,
			);
			await invoke('install_mod_version', {
				rmod: mod,
				version,
			});

			// Update the mod's installed version and notify the user of the success
			mod.installedVersion = version;
			notify.success(
				'Mod installed',
				`${mod.name} v${version.semver} was successfully installed.`,
			);
		} catch (err) {
			// Notify the user of the failure
			notify.error(
				'Error installing mod',
				`Error installing ${mod.name} v${version.semver}:\n${err}`,
			);
			throw err;
		} finally {
			// Clear the operation for the mod
			operations[mod.id] = null;
		}
	}

	/**
	 * Requests the uninstallation of a mod from the backend and displays an alert when a result is received
	 * @param {ResoluteMod|string} mod
	 */
	async function uninstall(mod) {
		mod = typeof mod === 'string' ? mods.value[mod] : mod;
		const version = mod.installedVersion;

		try {
			// Add an operation for the mod being installed and request the installation from the backend
			operations[mod.id] = 'uninstall';
			await info(
				`Requesting uninstallation of mod ${mod.name} v${version.semver}`,
			);
			await invoke('uninstall_mod', { rmod: mod });

			// Update the mod's installed version and notify the user of the success
			mod.installedVersion = null;
			notify.success(
				'Mod uninstalled',
				`${mod.name} v${version.semver} was successfully uninstalled.`,
			);
		} catch (err) {
			// Notify the user of the failure
			notify.error(
				'Error uninstalling mod',
				`Error uninstalling ${mod.name} v${version.semver}:\n${err}`,
			);
			throw err;
		} finally {
			// Clear the operation for the mod
			operations[mod.id] = null;
		}
	}

	/**
	 * Requests the replacement of a mod version from the backend and displays an alert when a result is received
	 * @param {ResoluteMod|string} mod
	 * @param {ModVersion|string} [version] Version to update to (defaults to latest available)
	 * @param {boolean} [alert=true] Whether to alert the user for a result
	 */
	async function update(mod, version, alert = true) {
		mod = typeof mod === 'string' ? mods.value[mod] : mod;
		const oldVersion = mod.installedVersion;

		// Determine the version to request the update to
		if (!version) version = mod.latestVersion;
		else if (typeof version === 'string') version = mod.versions[version];

		try {
			// Add an operation for the mod being installed and request the installation from the backend
			operations[mod.id] = 'update';
			await info(
				`Requesting replacement of mod ${mod.name} v${version.semver} with v${oldVersion.semver}`,
			);
			await invoke('replace_mod_version', {
				rmod: mod,
				version,
			});

			// Update the mod's installed version and notify the user of the success
			mod.installedVersion = version;
			if (alert) {
				const action = semverLt(version.semver, oldVersion.semver)
					? 'downgraded'
					: 'updated';
				notify.success(
					`Mod ${action}`,
					`${mod.name} v${oldVersion.semver} was successfully ${action} to ${version.semver}.`,
				);
			}
		} catch (err) {
			// Notify the user of the failure
			if (alert) {
				const action = semverLt(version.semver, oldVersion.semver)
					? 'downgrading'
					: 'updating';
				notify.error(
					`Error ${action} mod`,
					`Error ${action} ${mod.name} v${oldVersion.semver} to v${version.semver}:\n${err}`,
				);
			}
			throw err;
		} finally {
			// Clear the operation for the mod
			operations[mod.id] = null;
		}
	}

	/**
	 * Requests discovery of installed mods from the backend and alerts the user to the result
	 * @returns {Object} {@link ResoluteMod}s mapped by their ID
	 */
	async function discover() {
		if (discovering.value) {
			throw new Error('Already discovering installed mods.');
		}
		discovering.value = true;

		try {
			await info('Requesting installed mod discovery');
			const mods = await invoke('discover_installed_mods');
			for (const id of Object.keys(mods)) mods[id] = new ResoluteMod(mods[id]);

			console.debug('Installed mods discovered', mods);
			info(`${Object.keys(mods).length} mods discovered`);

			this.$patch({ mods });
			return mods;
		} catch (err) {
			error(`Error discovering installed mods: ${err}`);
			notify.error(
				'Error discovering mods',
				`Error discovering installed mods:\n${err}`,
			);
			throw err;
		} finally {
			discovering.value = false;
		}
	}

	/**
	 * Check whether a mod is being operated on, and thus actions for it should be disabled
	 * @param {ResoluteMod|string} mod
	 */
	function isBusy(mod) {
		mod = typeof mod === 'object' ? mod.id : mod;
		return Boolean(operations?.[mod]);
	}

	/**
	 * Check whether a mod is being installed
	 * @param {ResoluteMod|string} mod
	 */
	function isInstalling(mod) {
		mod = typeof mod === 'object' ? mod.id : mod;
		return operations?.[mod] === 'install';
	}

	/**
	 * Check whether a mod is being uninstalled
	 * @param {ResoluteMod|string} mod
	 */
	function isUninstalling(mod) {
		mod = typeof mod === 'object' ? mod.id : mod;
		return operations?.[mod] === 'uninstall';
	}

	/**
	 * Check whether a mod is being updated
	 * @param {ResoluteMod|string} mod
	 */
	function isUpdating(mod) {
		mod = typeof mod === 'object' ? mod.id : mod;
		return operations?.[mod] === 'update';
	}

	return {
		mods,
		operations,
		load,
		loadInstalled,
		discover,
		loading,
		loadingInstalled,
		discovering,
		hasLoaded,
		hasLoadedInstalled,
		install,
		uninstall,
		update,
		isBusy,
		isInstalling,
		isUninstalling,
		isUpdating,
	};
});

export default useModStore;
