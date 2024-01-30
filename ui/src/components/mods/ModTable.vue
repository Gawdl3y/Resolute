<template>
	<v-data-table
		ref="dataTable"
		:headers
		:items
		item-key="id"
		:items-per-page="settings.current[modsPerPageSetting]"
		:loading
		:search="filter"
		filter-mode="some"
		:group-by="groupBy"
		:no-data-text="noDataText"
		fixed-header
		hover
		@update:items-per-page="onItemsPerPageUpdate"
	>
		<template #item="{ item: mod }">
			<tr
				tabindex="0"
				class="mod-table-row"
				:class="{ deprecated: mod.isDeprecated }"
				@click="emit('showModDetails', mod)"
			>
				<td v-if="groupBy"></td>
				<!-- eslint-disable vue/no-v-html -->
				<td class="mod-name" v-html="wrappableCamelCase(mod.name)"></td>
				<!-- eslint-enable vue/no-v-html -->
				<td>{{ mod.description }}</td>
				<td v-if="!groupBy">{{ mod.category }}</td>
				<td class="mod-version"><ModVersionStatus :mod /></td>
				<td>
					<div class="d-flex flex-nowrap justify-end">
						<ModUninstaller
							v-if="mod.installedVersion"
							v-slot="{ uninstall, uninstalling, busy }"
							:mod
						>
							<IconButton
								:icon="mdiDelete"
								:disabled="disabled || (busy && !uninstalling)"
								:loading="uninstalling"
								tooltip="Uninstall"
								variant="plain"
								density="comfortable"
								@click.stop="uninstall"
							/>
						</ModUninstaller>

						<ModInstaller
							v-if="!mod.hasUpdate && !mod.isUnrecognized"
							v-slot="{ install, installing, busy }"
							:mod
						>
							<IconButton
								:icon="mod.installedVersion ? mdiRefresh : mdiDownload"
								:disabled="disabled || (busy && !installing)"
								:loading="installing"
								:tooltip="mod.installedVersion ? 'Reinstall' : 'Install'"
								variant="plain"
								density="comfortable"
								@click.stop="install"
							/>
						</ModInstaller>

						<ModUpdater
							v-else-if="mod.hasUpdate"
							v-slot="{ update, updating, busy }"
							:mod
						>
							<IconButton
								:icon="mdiUpdate"
								:disabled="disabled || (busy && !updating)"
								:loading="updating"
								tooltip="Update"
								variant="plain"
								density="comfortable"
								@click.stop="update"
							/>
						</ModUpdater>
					</div>
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
					:ref="addGroupHeader"
					:colspan="columns.length"
					:data-open="isGroupOpen(item)"
					:data-group="item.value"
					style="cursor: pointer"
					@click="toggleGroup(item)"
				>
					<IconButton
						:icon="isGroupOpen(item) ? '$expand' : '$next'"
						:tooltip="isGroupOpen(item) ? 'Collapse' : 'Expand'"
						variant="text"
						size="small"
						@click.stop="toggleGroup(item)"
					/>

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
import { ref, computed, onBeforeUpdate } from 'vue';
import { mdiDownload, mdiDelete, mdiUpdate, mdiRefresh } from '@mdi/js';

import { wrappableCamelCase } from '../../util';
import useSettings from '../../composables/settings';
import ModVersionStatus from './ModVersionStatus.vue';
import ModInstaller from './ModInstaller.vue';
import ModUninstaller from './ModUninstaller.vue';
import ModUpdater from './ModUpdater.vue';
import IconButton from '../IconButton.vue';

const props = defineProps({
	mods: { type: Object, default: null },
	disabled: { type: Boolean, default: false },
	loading: { type: Boolean, default: false },
	grouped: { type: Boolean, default: true },
	noDataText: { type: String, default: undefined },
});
const emit = defineEmits(['showModDetails']);
defineExpose({ expandAllGroups, collapseAllGroups });

const settings = useSettings();

/**
 * Headers for the data table - automatically adjusted based on whether mods should be grouped
 */
const headers = computed(() => {
	const headers = [
		{ title: 'Name', key: 'name' },
		{ title: 'Description', key: 'description' },
		{ title: 'Category', key: 'category' },
		{ title: 'Version', key: 'sortableVersionStatus', filterable: false },
		{ title: null, key: 'tags', sortable: false, filter: filterItem },
	];

	// If the mods should be grouped, ditch the category header
	if (props.grouped) {
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
	return props.grouped ? [{ key: 'category', order: 'asc' }] : undefined;
});

/**
 * Text to filter the table with
 */
const filter = ref(null);

/**
 * Custom filter function used for the fake tags column - searches tags, authors, and category
 * @param {?string[]} value Categories of of the item
 * @param {?string} query Search string
 * @param {Object} item Data table item object
 */
function filterItem(value, query, item) {
	if (!query) return false;
	query = query.toLowerCase();
	return (
		value?.join?.(' ')?.toLowerCase?.()?.includes?.(query) ||
		item.raw.category.toLowerCase().includes(query) ||
		item.raw.authors.some((author) => author.name.toLowerCase().includes(query))
	);
}

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

/**
 * Group header cells that have been added by the ref function {@link addGroupHeader}
 */
let groupHeaders = [];

// Need to clear the group header cells whenever the component is doing an update
onBeforeUpdate(() => {
	groupHeaders = [];
});

/**
 * Adds a group header cell to the list of cells if there isn't already one for that group
 * @param {HTMLTableCellElement} header
 */
function addGroupHeader(header) {
	if (!header) return;

	const group = header.getAttribute('data-group');
	const alreadyExists = groupHeaders.some(
		(header) => header.getAttribute('data-group') === group,
	);
	if (alreadyExists) return;

	groupHeaders.push(header);
}

/**
 * Expands any collapsed group headers
 */
function expandAllGroups() {
	for (const header of groupHeaders) {
		if (header.getAttribute('data-open') === 'false') header.click();
	}
}

/**
 * Collapses any expanded group headers
 */
function collapseAllGroups() {
	for (const header of groupHeaders) {
		if (header.getAttribute('data-open') === 'true') header.click();
	}
}
</script>

<style>
.mod-table-row {
	cursor: pointer;
}

.mod-table-row.deprecated {
	background: rgba(var(--v-theme-error), var(--v-selected-opacity));
}

.mod-table-row .mod-name {
	min-width: 12em;
	max-width: 16em;
	overflow-wrap: break-word;
}

.mod-table-row .mod-version {
	width: 7em;
}
</style>
