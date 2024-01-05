<template>
	<ModsPage
		title="Mod Index"
		no-data-text="No mod data is available."
		:mods="modStore.mods"
		:load-mods="loadMods"
		:grouped="settings.current.groupModIndex"
	/>
</template>

<script setup>
import useSettings from '../../composables/settings';
import useModStore from '../../stores/mods';
import ModsPage from './ModsPage.vue';

const settings = useSettings();
const modStore = useModStore();

/**
 * Loads all mods from the backend
 * @param {boolean} [bypassCache=false] Whether to bypass the manifest cache
 */
async function loadMods(bypassCache = false) {
	if (modStore.loading) return;
	await modStore.load(bypassCache);
}
</script>
