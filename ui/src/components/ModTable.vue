<template>
	<v-data-table
		:headers="headers"
		:items="items"
		item-key="id"
		:items-per-page="25"
		:loading="!mods"
	>
		<template #item="{ item: mod }">
			<tr>
				<td style="max-width: 14em; overflow-wrap: break-word">
					{{ mod.name }}
				</td>
				<td>{{ mod.description }}</td>
				<td>{{ mod.category }}</td>
				<td>
					<v-tooltip text="Install" :open-delay="500">
						<template #activator="{ props: activator }">
							<v-btn
								:icon="mdiDownload"
								variant="plain"
								v-bind="activator"
								@click="installMod(mod)"
							/>
						</template>
					</v-tooltip>
				</td>
			</tr>
		</template>
	</v-data-table>
</template>

<script setup>
import { computed } from 'vue';
import { compare as semverCompare } from 'semver';
import { invoke } from '@tauri-apps/api';
import { message } from '@tauri-apps/api/dialog';
import { mdiDownload } from '@mdi/js';

const props = defineProps({
	mods: { type: Object, default: null },
});

const headers = [
	{ title: 'Name', key: 'name' },
	{ title: 'Description', key: 'description' },
	{ title: 'Category', key: 'category' },
	{ title: '', sortable: false },
];
const items = computed(() => (props.mods ? Object.values(props.mods) : []));

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
