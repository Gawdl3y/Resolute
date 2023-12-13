<template>
	<v-data-table
		:headers="headers"
		:items="items"
		item-key="id"
		:items-per-page="25"
		:loading="!mods"
		:search="filter"
		:group-by="groupBy"
		fixed-header
	>
		<template #item="{ item: mod }">
			<tr>
				<td v-if="groupBy"></td>
				<td style="max-width: 14em; overflow-wrap: break-word">
					{{ mod.name }}
				</td>
				<td>{{ mod.description }}</td>
				<td v-if="!groupBy">{{ mod.category }}</td>
				<td>
					<v-tooltip text="Install" :open-delay="500">
						<template #activator="{ props: activator }">
							<v-btn
								:icon="mdiDownload"
								:disabled="disabled"
								variant="plain"
								v-bind="activator"
								@click="installMod(mod)"
							/>
						</template>
					</v-tooltip>
				</td>
			</tr>
		</template>

		<!-- eslint-disable-next-line vue/valid-v-slot -->
		<template #header.data-table-group>
			<div style="width: 2em"></div>
		</template>

		<template #group-header="{ item, columns, toggleGroup, isGroupOpen }">
			<tr>
				<td
					:colspan="columns.length"
					style="cursor: pointer"
					@click="toggleGroup(item)"
				>
					<v-tooltip
						:text="isGroupOpen(item) ? 'Collapse' : 'Expand'"
						:open-delay="500"
					>
						<template #activator="{ props: activator }">
							<v-btn
								size="small"
								variant="text"
								:icon="isGroupOpen(item) ? '$expand' : '$next'"
								v-bind="activator"
								@click.stop="toggleGroup(item)"
							/>
						</template>
					</v-tooltip>

					{{ item.value }} ({{ item.items.length }})
				</td>
			</tr>
		</template>

		<!-- eslint-disable-next-line vue/valid-v-slot -->
		<template #footer.prepend>
			<v-text-field
				v-model="filter"
				label="Filter"
				density="compact"
				variant="outlined"
				clearable
				hide-details
				class="ms-1 me-auto"
				style="max-width: 25%"
			/>
		</template>
	</v-data-table>
</template>

<script setup>
import { ref, computed } from 'vue';
import { compare as semverCompare } from 'semver';
import { invoke } from '@tauri-apps/api';
import { message } from '@tauri-apps/api/dialog';
import { mdiDownload } from '@mdi/js';

import useSettings from '../settings';

const props = defineProps({
	mods: { type: Object, default: null },
	disabled: { type: Boolean, default: false },
});
const settings = useSettings();

const headers = computed(() => {
	const headers = [
		{ title: 'Name', key: 'name' },
		{ title: 'Description', key: 'description' },
		{ title: 'Category', key: 'category' },
		{ title: null, sortable: false },
	];

	if (settings.current.groupMods) {
		const categoryIdx = headers.findIndex((head) => head.key === 'category');
		headers.splice(categoryIdx, 1);
	}

	return headers;
});
const groupBy = computed(() =>
	settings.current.groupMods ? [{ key: 'category', order: 'asc' }] : undefined,
);
const items = computed(() => (props.mods ? Object.values(props.mods) : []));
const filter = ref(null);

/**
 * Requests the installation of a mod from the backend and displays an alert when a result is received
 * @param {Object} mod Raw mod data
 */
async function installMod(mod) {
	// Determine the latest version
	const versions = Object.values(mod.versions);
	versions.sort((ver1, ver2) => semverCompare(ver2.semver, ver1.semver));
	const version = versions[0];

	// Request the version install from the backend and display an alert for the result
	try {
		await invoke('install_version', {
			rmod: mod,
			version,
		});
		await message(
			`${mod.name} v${version.semver} was successfully installed.`,
			{
				title: 'Mod installed',
				type: 'info',
			},
		);
	} catch (err) {
		await message(`Error installing ${mod.name} v${version.semver}:\n${err}`, {
			title: 'Error installing mod',
			type: 'error',
		});
	}
}
</script>
