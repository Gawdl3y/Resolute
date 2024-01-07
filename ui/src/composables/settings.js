import { reactive } from 'vue';
import { invoke } from '@tauri-apps/api';
import { Store } from 'tauri-plugin-store-api';
import { info } from 'tauri-plugin-log-api';

let store;
let storeUnlisten;
const currentSettings = reactive({
	resonitePath: null,
	manifestUrl: null,
	connectTimeout: 10,
	theme: null,
	groupModIndex: true,
	modsPerPageGrouped: -1,
	modsPerPageUngrouped: 25,
	modAuthorTools: false,
	setupGuideDone: false,
	allowClosingSetupGuide: false,
	modsAutodiscovered: false,
});

export function useSettings() {
	/**
	 * Initializes the store and propagates changes automatically
	 */
	async function init() {
		if (store) {
			info('Tearing down old settings store');
			storeUnlisten();
		}

		// Set up the store and change listener
		info('Initializing settings store');
		store = new Store('.settings.dat');
		storeUnlisten = await store.onChange((key, val) => {
			currentSettings[key] = val;
		});

		// Load all existing settings
		const entries = await store.entries();
		for (const [key, val] of entries) currentSettings[key] = val;
	}

	/**
	 * Stores a setting value in the settings store
	 * @param {string} setting
	 * @param {*} value
	 * @param {boolean} [persistNow=true] Whether to immediately save the change to disk
	 */
	async function set(setting, value, persistNow = true) {
		await store.set(setting, value);
		info(`Setting ${setting} set to ${value}, persistNow = ${persistNow}`);
		if (persistNow) await persist();

		if (setting === 'resonitePath') invoke('resonite_path_changed');
		else if (setting === 'connectTimeout') invoke('connect_timeout_changed');
	}

	/**
	 * Deletes a setting value from the settings store
	 * @param {string} setting
	 * @param {boolean} [persistNow=true] Whether to immediately save the change to disk
	 */
	async function unset(setting, persistNow = true) {
		await store.delete(setting);
		info(`Setting ${setting} unset, persistNow = ${persistNow}`);
		if (persistNow) await persist();
	}

	/**
	 * Persists all changes to settings made with set/unset to disk
	 */
	async function persist() {
		await store.save();
		info('Settings persisted to disk');
	}

	return {
		current: currentSettings,
		init,
		set,
		unset,
		persist,
	};
}

export default useSettings;
