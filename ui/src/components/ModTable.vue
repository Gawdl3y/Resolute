<template>
	<v-data-table
		:headers="headers"
		:items="items"
		item-key="id"
		:items-per-page="25"
		:loading="loading"
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
				<td><ModVersionStatus :mod="mod" /></td>
				<td>
					<ModInstaller v-slot="{ install, installing, busy }" :mod="mod.id">
						<v-tooltip text="Install" :open-delay="500">
							<template #activator="{ props: activator }">
								<v-btn
									:icon="mdiDownload"
									:disabled="disabled || (busy && !installing)"
									:loading="installing"
									variant="plain"
									v-bind="activator"
									@click="install"
								/>
							</template>
						</v-tooltip>
					</ModInstaller>
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
import { mdiDownload } from '@mdi/js';

import useSettings from '../composables/settings';
import ModVersionStatus from './ModVersionStatus.vue';
import ModInstaller from './ModInstaller.vue';

const props = defineProps({
	mods: { type: Object, default: null },
	disabled: { type: Boolean, default: false },
	loading: { type: Boolean, default: false },
});
const settings = useSettings();

const headers = computed(() => {
	const headers = [
		{ title: 'Name', key: 'name' },
		{ title: 'Description', key: 'description' },
		{ title: 'Category', key: 'category' },
		{ title: 'Version', key: 'sortableVersionStatus' },
		{ title: null, sortable: false },
	];

	// If the mods should be grouped, ditch the category header
	if (settings.current.groupMods) {
		const categoryIdx = headers.findIndex((head) => head.key === 'category');
		headers.splice(categoryIdx, 1);
	}

	return headers;
});

const items = computed(() => (props.mods ? Object.values(props.mods) : []));

const groupBy = computed(() =>
	settings.current.groupMods ? [{ key: 'category', order: 'asc' }] : undefined,
);

const filter = ref(null);
</script>
