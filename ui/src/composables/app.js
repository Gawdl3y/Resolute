import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

let name = ref('');
let identifier = ref('');
let version = ref('');
let tauriVersion = ref('');
let debug = ref(false);

export function useApp() {
	/**
	 * Retrieves the app info from the backend
	 * @returns {Object}
	 */
	async function init() {
		const info = await invoke('get_app_info');
		console.debug('App information retrieved:', info);

		name.value = info.name;
		identifier.value = info.identifier;
		version.value = info.version;
		tauriVersion.value = info.tauriVersion;
		debug.value = info.debug;

		return info;
	}

	return {
		init,
		name,
		identifier,
		version,
		tauriVersion,
		debug,
	};
}

export default useApp;
