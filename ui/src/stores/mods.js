import { ref, reactive } from 'vue';
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api';
import { message } from '@tauri-apps/api/dialog';
import { info, error } from 'tauri-plugin-log-api';

import { ResoluteMod } from '../structs/mod';

export const useModStore = defineStore('mods', () => {
	const mods = ref(null);
	const loading = ref(false);
	const loadingInstalled = ref(false);
	const operations = reactive({});

	/**
	 * Retrieves mod data from the backend
	 * @param {boolean} [bypassCache=false] Whether to bypass the manifest cache when possible
	 * @param {boolean} [alert=true] Whether to alert the user for failures
	 * @returns {Object} Raw mod data
	 */
	async function load(bypassCache = false, alert = true) {
		if (loading.value) throw new Error('Already loading mods.');
		loading.value = true;

		try {
			await info(`Requesting mod load, bypassCache = ${bypassCache}`);
			const mods = await invoke('load_all_mods', { bypassCache });
			for (const id of Object.keys(mods)) mods[id] = new ResoluteMod(mods[id]);

			console.debug('Mods loaded', mods);
			info(`${Object.keys(mods).length} mods loaded`);

			this.$patch({ mods });
			return mods;
		} catch (err) {
			error(`Error loading mods: ${err}`);

			if (alert) {
				message(`Error loading mod list:\n${err}`, {
					title: 'Error loading mods',
					type: 'error',
				});
			}

			throw err;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Retrieves installed mod data from the backend
	 * @returns {Object} Raw mod data
	 */
	async function loadInstalled() {
		if (loadingInstalled.value) {
			throw new Error('Already loading installed mods.');
		}
		loadingInstalled.value = true;

		try {
			await info(`Requesting installed mod load`);
			const mods = await invoke('load_installed_mods');
			for (const id of Object.keys(mods)) mods[id] = new ResoluteMod(mods[id]);

			console.debug('Installed mods loaded', mods);
			info(`${Object.keys(mods).length} installed mods loaded`);

			this.$patch({ mods });
			return mods;
		} catch (err) {
			error(`Error loading installed mods: ${err}`);
			message(`Error loading installed mod list:\n${err}`, {
				title: 'Error loading mods',
				type: 'error',
			});
			throw err;
		} finally {
			loadingInstalled.value = false;
		}
	}

	/**
	 * Requests the installation of a mod from the backend and displays an alert when a result is received
	 * @param {string} modID
	 */
	async function install(modID) {
		const mod = mods.value[modID];
		const version = mod.latestVersion;

		try {
			// Add an operation for the mod being installed and request the installation from the backend
			operations[modID] = 'install';
			await info(
				`Requesting installation of mod ${mod.name} v${version.semver}`,
			);
			await invoke('install_mod_version', {
				rmod: mod,
				version,
			});

			// Update the mod's installed version and notify the user of the success
			mod.installedVersion = version;
			message(`${mod.name} v${version.semver} was successfully installed.`, {
				title: 'Mod installed',
				type: 'info',
			});
		} catch (err) {
			// Notify the user of the failure
			message(`Error installing ${mod.name} v${version.semver}:\n${err}`, {
				title: 'Error installing mod',
				type: 'error',
			});
			throw err;
		} finally {
			// Clear the operation for the mod
			operations[modID] = null;
		}
	}

	/**
	 * Check whether a mod is being operated on, and thus actions for it should be disabled
	 * @param {Object} modID
	 */
	function isBusy(modID) {
		return Boolean(operations?.[modID]);
	}

	/**
	 * Check whether a mod is being installed
	 * @param {Object} modID
	 */
	function isInstalling(modID) {
		return operations?.[modID] === 'install';
	}

	return {
		mods,
		operations,
		load,
		loadInstalled,
		install,
		isBusy,
		isInstalling,
	};
});

export default useModStore;
