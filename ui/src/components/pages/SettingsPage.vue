<template>
	<AppHeader title="Settings" />

	<v-main>
		<v-form>
			<v-container>
				<v-row>
					<v-col>
						<v-text-field
							v-model="settings.current.resonitePath"
							label="Resonite path"
							variant="solo"
							readonly
						>
							<template #append-inner>
								<v-tooltip text="Change folder" :open-delay="500">
									<template #activator="{ props }">
										<v-btn
											v-bind="props"
											:icon="mdiFolderSearch"
											variant="flat"
											@click="findResonitePath"
										/>
									</template>
								</v-tooltip>
							</template>
						</v-text-field>
					</v-col>
				</v-row>
			</v-container>
		</v-form>
	</v-main>
</template>

<script setup>
import { open, ask } from '@tauri-apps/api/dialog';
import { exists as fsExists } from '@tauri-apps/api/fs';
import { join as pathJoin } from '@tauri-apps/api/path';
import { mdiFolderSearch } from '@mdi/js';

import useSettings from '../../settings';
import AppHeader from '../AppHeader.vue';

const settings = useSettings();

async function findResonitePath() {
	let dir, exists;

	while (!exists) {
		// Prompt to choose a folder
		dir = await open({
			directory: true,
			defaultPath: settings.current.resonitePath,
		});
		if (!dir) return;

		// Verify the existence of the Resonite executable
		exists = await fsExists(await pathJoin(dir, 'Resonite.exe'));
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
