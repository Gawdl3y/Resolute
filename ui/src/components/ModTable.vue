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
import { invoke } from '@tauri-apps/api';
import { message } from '@tauri-apps/api/dialog';
import { info, error } from 'tauri-plugin-log-api';
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
	console.log('Triggering download', mod);

	try {
		await invoke('download_version', {
			version: Object.values(mod.versions)[0],
		});
		info(`Installed ${mod.name}`);
		message(`${mod.name} was successfully installed.`, {
			title: 'Mod installed',
			type: 'info',
		});
	} catch (err) {
		error(`Error installing ${mod.name}: ${err}`);
		message(`Error installing ${mod.name}:\n${err}`, {
			title: 'Error installing mod',
			type: 'error',
		});
	}
}
</script>
