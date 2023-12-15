<template>
	<slot
		:install="install"
		:busy="busy"
		:installing="installing"
		:installed="installed"
		:error="error"
	/>
</template>

<script setup>
import { ref, computed } from 'vue';

import useModStore from '../stores/mods';

const props = defineProps({ mod: { type: String, required: true } });
const emit = defineEmits(['install', 'error']);

const modStore = useModStore();
const busy = computed(() => modStore.isBusy(props.mod));
const installing = computed(() => modStore.isInstalling(props.mod));
const installed = ref(false);
const error = ref(null);

/**
 * Installs the mod and updates internal state as needed
 */
async function install() {
	try {
		await modStore.install(props.mod);
		installed.value = true;
		emit('install');
	} catch (err) {
		error.value = err;
		emit('error', err);
		throw error;
	}
}
</script>
