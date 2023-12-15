<template>
	<v-text-field
		v-model="settings.current.resonitePath"
		label="Resonite path"
		:variant="variant"
		readonly
	>
		<template #append-inner>
			<v-tooltip text="Change folder" :open-delay="500">
				<template #activator="{ props: activator }">
					<v-btn
						v-bind="activator"
						:icon="mdiFolderSearch"
						variant="text"
						@click="openPathChooser"
					/>
				</template>
			</v-tooltip>
		</template>
	</v-text-field>
</template>

<script setup>
import { open, ask } from '@tauri-apps/api/dialog';
import { exists as fsExists } from '@tauri-apps/api/fs';
import { join as pathJoin } from '@tauri-apps/api/path';
import { mdiFolderSearch } from '@mdi/js';

import useSettings from '../settings';

defineProps({ variant: { type: String, default: 'solo' } });
const settings = useSettings();

/**
 * Opens a dialog to choose a Resonite installation path and validates it, then saves the setting when confirmed
 */
async function openPathChooser() {
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

	await settings.store.set('resonitePath', dir);
	await settings.store.save();
}
</script>
