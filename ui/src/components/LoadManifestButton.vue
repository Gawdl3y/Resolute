<template>
	<el-button @click.prevent="loadManifest">
		<template #icon v-if="$slots.icon"><slot name="icon" /></template>
		<slot />
	</el-button>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri';

const props = defineProps({ bypassCache: Boolean })

async function loadManifest() {
	console.info(await invoke('load_manifest', { bypassCache: props.bypassCache }));
}
</script>
