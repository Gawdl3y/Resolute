<template>
	<AppHeader title="Settings" />

	<v-main>
		<v-form>
			<v-container>
				<v-row>
					<v-col>
						<!-- Resonite path setting -->
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

						<!-- Custom manifest URL setting -->
						<v-text-field
							v-model="settings.current.manifestUrl"
							label="Custom manifest URL"
							variant="solo"
							:rules="[rules.url]"
							@blur="saveManifestUrl"
							@keypress.enter="saveManifestUrl"
						/>

						<!-- Theme setting -->
						<v-select
							v-model="settings.current.theme"
							label="Theme"
							variant="solo"
							:items="themes"
							item-title="name"
							item-value="val"
							@update:model-value="saveTheme"
						/>
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

/**
 * Validation rules
 */
const rules = {
	url(val) {
		if (!val) return true;

		try {
			return Boolean(new URL(val));
		} catch (_err) {
			return 'Invalid URL';
		}
	},
};

/**
 * Opens a dialog to choose a Resonite installation path and validates it, then saves the setting when confirmed
 */
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

/**
 * Saves the custom manifest URL if valid
 */
function saveManifestUrl() {
	const url = settings.current.manifestUrl;
	const valid = rules.url(url);
	if (!valid || typeof valid === 'string') return;

	if (url) settings.store.set('manifestUrl', url);
	else settings.store.delete('manifestUrl');
	settings.store.save();
}

/**
 * Theme choices
 */
const themes = [
	{ name: 'System', val: null },
	{ name: 'Light', val: 'light' },
	{ name: 'Dark', val: 'dark' },
];

/**
 * Saves the theme selection
 */
function saveTheme() {
	settings.store.set('theme', settings.current.theme);
	settings.store.save();
}
</script>
