<template>
	<ModsPage
		title="Installed Mods"
		:mods="mods"
		:load-mods="loadMods"
		:allow-grouping="false"
	/>
</template>

<script setup>
import { computed } from 'vue';

import useModStore from '../../stores/mods';
import ModsPage from './ModsPage.vue';

const modStore = useModStore();
const mods = computed(() => {
	if (!modStore.mods) return modStore.mods;

	const mods = {};
	for (const mod of Object.values(modStore.mods)) {
		if (!mod.installedVersion) continue;
		mods[mod.id] = mod;
	}

	return mods;
});

/**
 * Loads installed mods first, then loads all mods from the manifest to fill in any updated data
 * @param {boolean} [bypassCache=false] Whether to bypass the manifest cache
 */
async function loadMods(bypassCache = false) {
	await modStore.loadInstalled();
	await modStore.load(bypassCache, false).catch(() => {});
}
</script>
