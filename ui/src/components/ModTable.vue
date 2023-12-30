<template>
	<v-data-table
		:headers="headers"
		:items="items"
		item-key="id"
		:items-per-page="settings.current[modsPerPageSetting]"
		:loading="loading"
		:search="filter"
		:group-by="groupBy"
		fixed-header
		@update:items-per-page="onItemsPerPageUpdate"
	>
		<template #item="{ item: mod }">
			<tr>
				<td v-if="groupBy"></td>
				<td style="max-width: 14em; overflow-wrap: break-word">
					{{ mod.name }}
				</td>
				<td>{{ mod.description }}</td>
				<td v-if="!groupBy">{{ mod.category }}</td>
				<td style="width: 7em"><ModVersionStatus :mod="mod" /></td>
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
	allowGrouping: { type: Boolean, default: true },
});
const settings = useSettings();

/**
 * Headers for the data table - automatically adjusted based on whether mods should be grouped
 */
const headers = computed(() => {
	const headers = [
		{ title: 'Name', key: 'name' },
		{ title: 'Description', key: 'description' },
		{ title: 'Category', key: 'category' },
		{ title: 'Version', key: 'sortableVersionStatus' },
		{ title: null, sortable: false },
	];

	// If the mods should be grouped, ditch the category header
	if (props.allowGrouping && settings.current.groupModIndex) {
		const categoryIdx = headers.findIndex((head) => head.key === 'category');
		headers.splice(categoryIdx, 1);
	}

	return headers;
});

/**
 * Items for the data table
 */
const items = computed(() => (props.mods ? Object.values(props.mods) : []));

/**
 * groupBy parameter for the data table - automatically adjusted based on whether mods should be grouped
 */
const groupBy = computed(() => {
	if (!props.allowGrouping) return undefined;
	return settings.current.groupModIndex
		? [{ key: 'category', order: 'asc' }]
		: undefined;
});

/**
 * Text to filter the table with
 */
const filter = ref(null);

/**
 * Setting key to use for the itemsPerPage parameter on the table
 */
const modsPerPageSetting = computed(
	() => `modsPerPage${groupBy.value ? 'Grouped' : 'Ungrouped'}`,
);

/**
 * Handles an update to the itemsPerPage selection on the table
 * @param {number} itemsPerPage
 */
function onItemsPerPageUpdate(itemsPerPage) {
	settings.set(modsPerPageSetting.value, itemsPerPage);
}
</script>
