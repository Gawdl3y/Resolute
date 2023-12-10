<template>
	<el-auto-resizer>
		<template #default="{ width, height }">
			<el-table :data="data" :width="width" :height="height">
				<el-table-column prop="name" label="Name" />
				<el-table-column prop="description" label="Description" />
				<el-table-column>
					<template #default="scope">
						<el-button @click="handleInstall(scope.row)">Install</el-button>
					</template>
				</el-table-column>
			</el-table>
		</template>
	</el-auto-resizer>
</template>

<script setup>
import { invoke } from '@tauri-apps/api';
import { info, error } from 'tauri-plugin-log-api';
import { computed } from 'vue';
import { ElNotification } from 'element-plus';

const props = defineProps({ mods: Object });
const data = computed(() => props.mods ? Object.values(props.mods) : null);

async function handleInstall(mod) {
	console.log('Triggering download', mod);

	try {
		await invoke('download_version', { version: Object.values(mod.versions)[0] });
		info(`Installed ${mod.name}`);
		ElNotification({
			type: 'success',
			title: `Installed ${mod.name}`,
		});
	} catch(err) {
		error(`Error installing ${mod.name}: ${err}`);
		ElNotification({
			type: 'error',
			title: `Error installing ${mod.name}`,
			message: err.message,
			duration: 0,
		});
	}
}
</script>
