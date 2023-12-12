import { ref } from 'vue';
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api';
import { info, error } from 'tauri-plugin-log-api';

export default defineStore('mods', () => {
	const mods = ref(null);

	/**
	 * Retrieves mod data from the backend
	 * @param {boolean} [bypassCache=false] Whether to bypass the manifest cache when possible
	 * @returns {Object} Raw mod data
	 */
	async function load(bypassCache = false) {
		try {
			const mods = await invoke('load_manifest', { bypassCache });
			console.debug('Mods loaded', mods);
			info('Mods loaded');
			this.$patch({ mods });
			return mods;
		} catch (err) {
			error(`Error loading mods on the frontend: ${err}`);
			throw err;
		}
	}

	return { mods, load };
});
