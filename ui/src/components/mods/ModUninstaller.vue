<template>
	<slot
		:uninstall="uninstall"
		:busy="busy"
		:uninstalling="uninstalling"
		:uninstalled="uninstalled"
		:error="error"
	/>
</template>

<script setup>
import { ref, computed } from 'vue';
import { ask } from '@tauri-apps/api/dialog';

import useModStore from '../../stores/mods';

const props = defineProps({
	mod: { type: Object, required: true },
	confirm: { type: Boolean, default: true },
});
const emit = defineEmits(['uninstall', 'error']);

const modStore = useModStore();
const busy = computed(() => modStore.isBusy(props.mod));
const uninstalling = computed(() => modStore.isUninstalling(props.mod));
const uninstalled = ref(false);
const error = ref(null);

/**
 * Uninstalls the mod and updates internal state as needed
 */
async function uninstall() {
	if (props.confirm) {
		const answer = await ask(
			`Are you sure you want to uninstall ${props.mod.name} v${props.mod.installedVersion.semver}?`,
			{ title: 'Uninstalling mod', type: 'info' },
		);
		if (!answer) return;
	}

	try {
		await modStore.uninstall(props.mod);
		uninstalled.value = true;
		emit('uninstall');
	} catch (err) {
		error.value = err;
		emit('error', err);
		throw error;
	}
}
</script>
