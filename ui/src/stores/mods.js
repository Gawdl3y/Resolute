import { ref } from 'vue';
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api';

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
			this.$patch({ mods });
			return mods;
		} catch (err) {
			console.error('Error loading mods', err);
			throw err;
		}
	}

	return { mods, load };
});
