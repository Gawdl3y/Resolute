<template>
	<v-text-field
		v-model="settings.current.resonitePath"
		label="Resonite path"
		:variant
		readonly
	>
		<template #append-inner>
			<IconButton
				:icon="mdiAutoFix"
				tooltip="Autodetect"
				variant="text"
				@click="discoverPath"
			/>

			<IconButton
				:icon="mdiFolderSearch"
				tooltip="Choose folder"
				variant="text"
				@click="choosePath"
			/>
		</template>
	</v-text-field>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core';
import { open, ask } from '@tauri-apps/plugin-dialog';
import { exists as fsExists } from '@tauri-apps/plugin-fs';
import { join as pathJoin } from '@tauri-apps/api/path';
import { mdiFolderSearch, mdiAutoFix } from '@mdi/js';

import useNotifications from '../../composables/notifications';
import useSettings from '../../composables/settings';
import IconButton from '../IconButton.vue';

defineProps({ variant: { type: String, default: 'solo' } });
const notify = useNotifications();
const settings = useSettings();

/**
 * Opens a dialog to choose a Resonite installation path and validates it, then saves the setting when confirmed
 */
async function choosePath() {
	let dir, exists;

	while (!exists) {
		// Prompt to choose a folder
		dir = await open({
			directory: true,
			defaultPath: settings.current.resonitePath,
		});
		if (!dir) return;

		// Verify the existence of the Resonite executable (Windows or Linux)
		exists =
			(await fsExists(await pathJoin(dir, 'Resonite.exe'))) ||
			(await fsExists(await pathJoin(dir, 'Resonite.x86_64')));

		// If the executable doesn't exist, confirm they want to use this directory
		if (!exists) {
			const answer = await ask(
				"Couldn't locate the Resonite executable.\nSet this as the game path anyways?",
				{ title: 'No Resonite Executable', type: 'warning' },
			);
			if (answer) break;
		}
	}

	await settings.set('resonitePath', dir);
}

/**
 * Automatically detects a possible Resonite path, prompts the user to confirm using it if one is found, and saves it
 */
async function discoverPath() {
	try {
		// Try discovering a path
		const path = await invoke('discover_resonite_path');
		if (!path) {
			notify.info(
				'No Resonite Folder Found',
				'No Resonite folder could be automatically located. Please manually choose it instead.',
			);
			return;
		}

		// Confirm the user wants to use the discovered path
		const answer = await ask(
			`Found a Resonite folder:\n${path}\n\nUse this as the Resonite path?`,
			{ title: 'Found Resonite folder', type: 'info' },
		);
		if (answer) await settings.set('resonitePath', path);
	} catch (err) {
		notify.error(
			'Autodiscovery Error',
			`Error auto-discovering Resonite path:\n${err}`,
		);
	}
}
</script>
