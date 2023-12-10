<template>
	<el-button @click.prevent="loadManifest">
		<template #icon v-if="$slots.icon"><slot name="icon" /></template>
		<slot />
	</el-button>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri';
import useModStore from '../stores/mods';

const props = defineProps({ bypassCache: Boolean });
const emit = defineEmits(['load', 'error']);
const modStore = useModStore();

async function loadManifest() {
	try {
		const mods = await invoke('load_manifest', { bypassCache: props.bypassCache });
		console.debug('Mods loaded', mods);
		modStore.$patch({ mods });
		emit('load', mods);
	} catch(err) {
		console.error('Error loading mods', err);
		emit('error', err);
	}
}
</script>
