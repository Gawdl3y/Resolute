import { reactive } from 'vue';
import { Store } from 'tauri-plugin-store-api';
import { info } from 'tauri-plugin-log-api';

let store;
let storeUnlisten;
const settings = reactive({
	resonitePath: null,
});

function useSettings() {
	/**
	 * Initializes the store and propagates changes automatically
	 */
	async function init() {
		if(store) {
			info('Tearing down old settings store');
			storeUnlisten();
		}

		// Set up the store and change listener
		info('Initializing settings store');
		store = new Store('.settings.dat');
		storeUnlisten = await store.onChange((key, val) => { settings[key] = val; });

		// Load all existing settings
		const entries = await store.entries();
		for(const [key, val] of entries) settings[key] = val;
	}

	return {
		store,
		init,
		current: settings,
	};
}

export default useSettings;
