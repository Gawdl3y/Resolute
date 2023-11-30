<template>
	<el-container direction="vertical">
		<Header>
			<template #icon><i-ep-setting /></template>
			Settings
		</Header>

		<el-main>
			<el-form :model="settings.current">
				<el-form-item label="Resonite path" class="has-button">
					<el-input v-model="settings.current.resonitePath" />
					<el-button @click="findResonitePath">
						<template #icon><i-ep-folder /></template>
						Find
					</el-button>
				</el-form-item>
			</el-form>
		</el-main>
	</el-container>
</template>

<script setup>
import { open, ask } from '@tauri-apps/api/dialog';
import { exists as fsExists } from '@tauri-apps/api/fs';
import { join as pathJoin } from '@tauri-apps/api/path';

import useSettings from '../../settings';
import Header from '../Header.vue';

const settings = useSettings();

async function findResonitePath() {
	let dir, exists;

	while(!exists) {
		// Prompt to choose a folder
		dir = await open({
			directory: true,
			defaultPath: settings.current.resonitePath,
		});
		if(!dir) return;

		// Verify the existence of the Resonite executable
		exists = await fsExists(await pathJoin(dir, 'Resonite.exe'));
		if(!exists) {
			const answer = await ask(
				'Couldn\'t locate the Resonite executable.\nSet this as the game path anyways?',
				{ title: 'No Resonite Executable', type: 'warning' },
			);
			if(answer) break;
		}
	}

	await settings.store.set('resonitePath', dir);
}
</script>

<style>
.has-button .el-form-item__content {
	flex-wrap: nowrap;
	gap: 8px;
}
</style>
