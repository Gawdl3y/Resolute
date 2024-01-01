<template>
	<ModsPage
		title="Installed Mods"
		:mods="mods"
		:load-mods="loadMods"
		:allow-grouping="false"
	>
		<template #actions>
			<v-tooltip text="Update all" :open-delay="500">
				<template #activator="{ props: tooltipProps }">
					<v-btn
						:icon="mdiUpdate"
						:loading="modStore.operations.updateAll"
						:disabled="outdatedMods.length === 0"
						v-bind="tooltipProps"
						@click="updateAllMods()"
					/>
				</template>
			</v-tooltip>
		</template>
	</ModsPage>
</template>

<script setup>
import { computed } from 'vue';
import { message } from '@tauri-apps/api/dialog';
import { info, error } from 'tauri-plugin-log-api';
import { mdiUpdate } from '@mdi/js';

import useModStore from '../../stores/mods';
import ModsPage from './ModsPage.vue';

const modStore = useModStore();
const mods = computed(() => {
	if (!modStore.mods) return modStore.mods;

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
	try {
		await modStore.loadInstalled();
	} catch (err) {
		message(`Error loading installed mods:\n${err}`, {
			title: 'Error loading mods',
			type: 'error',
		});
	}

	try {
		await modStore.load(bypassCache, false);
	} catch (err) {
		message(`Error checking for updates:\n${err}`, {
			title: 'Error loading mods',
			type: 'error',
		});
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
		const promises = outdated.map((mod) => modStore.update(mod.id, false));
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
			await message(
				`The following mods were successfully updated:\n${succeededList}`,
				{
					title: 'Mods updated',
					type: 'info',
				},
			);
		}

		// Notify the user of any failures
		if (failed.length > 0) {
			const failedList = failed
				.map(({ mod, result }) => `${mod.name}:\n${result.reason}`)
				.join('\n\n');
			await message(`The following mods failed to update:\n\n${failedList}`, {
				title: 'Mod updates failed',
				type: 'error',
			});
		}
	} catch (err) {
		error(`Error batch-updating mods: ${err}`);
	} finally {
		modStore.operations.updateAll = false;
	}
}
</script>
