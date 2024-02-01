<template>
	<ModsPage
		title="Installed Mods"
		no-data-text="No mods have been installed yet."
		:mods
		:load-mods="loadMods"
		:grouped="false"
	>
		<template #actions="{ resonitePathExists }">
			<IconButton
				:icon="mdiToyBrickSearch"
				:loading="modStore.discovering"
				:disabled="!resonitePathExists"
				tooltip="Discover installed"
				@click="discoverInstalledMods"
			/>

			<IconButton
				:icon="mdiUpdate"
				:loading="modStore.operations.updateAll"
				:disabled="outdatedMods.length === 0"
				tooltip="Update all"
				@click="updateAllMods"
			/>
		</template>
	</ModsPage>
</template>

<script setup>
import { computed } from 'vue';
import { ask } from '@tauri-apps/plugin-dialog';
import { info, error } from '@tauri-apps/plugin-log';
import { mdiToyBrickSearch, mdiUpdate } from '@mdi/js';

import useNotifications from '../../composables/notifications';
import useModStore from '../../stores/mods';
import ModsPage from './ModsPage.vue';
import IconButton from '../IconButton.vue';

const notify = useNotifications();

const modStore = useModStore();
const mods = computed(() => {
	if (!modStore.mods || !modStore.hasLoadedInstalled) return null;

	const mods = {};
	for (const mod of Object.values(modStore.mods)) {
		if (!mod.installedVersion) continue;
		mods[mod.id] = mod;
	}

	return mods;
});
const outdatedMods = computed(() =>
	mods.value ? Object.values(mods.value).filter((mod) => mod.hasUpdate) : [],
);

/**
 * Loads installed mods first, then loads all mods from the manifest to fill in any updated data
 * @param {boolean} [bypassCache=false] Whether to bypass the manifest cache
 */
async function loadMods(bypassCache = false) {
	if (!modStore.loadingInstalled) {
		try {
			await modStore.loadInstalled();
		} catch (err) {
			notify.error(
				'Error loading mods',
				`Error loading installed mods:\n${err}`,
			);
		}
	}

	if (!modStore.loading) {
		try {
			await modStore.load(bypassCache, false);
		} catch (err) {
			notify.error('Error loading mods', `Error checking for updates:\n${err}`);
		}
	}
}

/**
 * Installs the latest version of all outdated mods
 */
async function updateAllMods() {
	info(`Batch-updating ${outdatedMods.value.length} mods`);
	modStore.operations.updateAll = true;
	const outdated = [...outdatedMods.value];

	try {
		// Request the update of every outdated mod
		const promises = outdated.map((mod) =>
			modStore.update(mod.id, null, false),
		);
		const results = await Promise.allSettled(promises);
		const updated = results.map((result, i) => ({
			mod: outdated[i],
			result,
		}));

		// Separate the results into successes and failures
		const succeeded = updated.filter(
			({ result }) => result.status === 'fulfilled',
		);
		const failed = updated.filter(({ result }) => result.status === 'rejected');

		console.debug('Batch update done', updated);
		info(
			`Batch update done, succeeded = ${succeeded.length}, failed = ${failed.length}`,
		);
		modStore.operations.updateAll = false;

		// Notify the user of any successes
		if (succeeded.length > 0) {
			const succeededList = succeeded
				.map(({ mod }) => `- ${mod.name} v${mod.installedVersion.semver}`)
				.join('\n');
			await notify.success(
				'Mods updated',
				`The following mods were successfully updated:\n${succeededList}`,
			);
		}

		// Notify the user of any failures
		if (failed.length > 0) {
			const failedList = failed
				.map(({ mod, result }) => `${mod.name}:\n${result.reason}`)
				.join('\n\n');
			await notify.error(
				'Mod updates failed',
				`The following mods failed to update:\n\n${failedList}`,
			);
		}
	} catch (err) {
		error(`Error batch-updating mods: ${err}`);
	} finally {
		modStore.operations.updateAll = false;
	}
}

/**
 * Discovers installed mods
 */
async function discoverInstalledMods() {
	const answer = await ask(
		`Are you sure you want to discover installed mods?\nThis should only be necessary when mods are manually installed outside of Resolute.`,
		{ title: 'Discovering installed mods', type: 'info' },
	);
	if (!answer) return;

	try {
		const mods = Object.values(await modStore.discover());
		const modList = mods
			.map((mod) => `- ${mod.name} v${mod.installedVersion.semver}`)
			.join('\n');
		notify.success(
			'Discovered mods',
			`Discovered ${mods.length} installed mods:\n${modList}`,
		);
	} catch (err) {
		console.error('Error discovering installed mods', err);
		notify.error(
			'Error discovering mods',
			`Error discovering installed mods:\n${err}`,
		);
	}
}
</script>
