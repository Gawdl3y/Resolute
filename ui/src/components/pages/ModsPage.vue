<template>
	<AppHeader :title>
		<template #actions>
			<slot name="actions" :resonite-path-exists />

			<IconButton
				v-if="grouped"
				:icon="expanded ? mdiArrowCollapseVertical : mdiArrowExpandVertical"
				:tooltip="`${expanded ? 'Collapse' : 'Expand'} all`"
				@click="toggleAllGroups"
			/>

			<IconButton
				:icon="mdiRefresh"
				:loading
				tooltip="Refresh mods"
				@click="loadModsFromFn(true)"
			/>
		</template>
	</AppHeader>

	<v-main v-resize="adjustTableHeight">
		<div ref="alerts">
			<v-alert
				v-if="!resonitePathExists"
				type="warning"
				:rounded="false"
				density="comfortable"
				class="rounded-0"
			>
				{{
					resonitePathExists === null
						? 'Please configure the Resonite path in the settings.'
						: "The configured Resonite path doesn't seem to exist. Please check the settings."
				}}
			</v-alert>

			<slot name="alerts" />
		</div>

		<ModTable
			ref="modTable"
			:mods
			:disabled="disabled || !resonitePathExists"
			:loading
			:style="`height: ${tableHeight}`"
			:grouped
			:no-data-text="noDataText"
			@show-mod-details="showModDetails"
		/>
	</v-main>

	<ModDetailsDialog
		v-if="route.params.mod && mods?.[route.params.mod]"
		:mod="mods[route.params.mod]"
		:disabled="disabled || !resonitePathExists"
		:close-on-back="false"
		@after-leave="closeModDetails"
	/>
</template>

<script setup>
import {
	ref,
	computed,
	watch,
	onBeforeMount,
	onMounted,
	onUnmounted,
} from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import {
	mdiArrowCollapseVertical,
	mdiArrowExpandVertical,
	mdiRefresh,
} from '@mdi/js';

import useSettings from '../../composables/settings';
import useModStore from '../../stores/mods';
import sidebarBus from '../../sidebar-bus';
import AppHeader from '../AppHeader.vue';
import ModTable from '../mods/ModTable.vue';
import ModDetailsDialog from '../mods/ModDetailsDialog.vue';
import IconButton from '../IconButton.vue';

defineExpose({
	adjustTableHeight,
	toggleAllGroups,
	showModDetails,
	closeModDetails,
});
const props = defineProps({
	title: { type: String, required: true },
	mods: { type: Object, default: null },
	loadMods: { type: Function, required: true },
	baseRoute: { type: String, required: true },
	disabled: { type: Boolean, default: false },
	grouped: { type: Boolean, default: true },
	noDataText: { type: String, default: undefined },
});

const router = useRouter();
const route = useRoute();

const settings = useSettings();
const modStore = useModStore();

const loading = ref(false);
const resonitePathExists = ref(true);

const alerts = ref(null);
const alertHeight = ref(0);
const tableHeight = computed(() => {
	if (!alerts.value) return '100%';
	return `calc(100% - ${alertHeight.value}px)`;
});

onMounted(() => {
	if (!props.mods) loadModsFromFn(false);
	sidebarBus.on('toggle', onSidebarToggle);

	// Automatically discover mods if it hasn't been done before and the setup guide has already been done
	const shouldAutodiscover =
		!settings.current.modsAutodiscovered2 &&
		settings.current.resonitePath &&
		settings.current.setupGuideDone &&
		!modStore.discovering;
	if (shouldAutodiscover) {
		modStore
			.discover()
			.then(() => {
				settings.set('modsAutodiscovered2', true);
			})
			.catch(() => {});
	}
});

onUnmounted(() => {
	if (resizeInterval) {
		clearInterval(resizeInterval);
		resizeInterval = null;
	}

	sidebarBus.off('toggle', onSidebarToggle);
});

// Validate the Resonite path on load and change
onBeforeMount(checkIfResonitePathExists);
watch(settings.current, checkIfResonitePathExists);

/**
 * Loads the mod data
 */
async function loadModsFromFn(bypassCache = false) {
	loading.value = true;

	try {
		await props.loadMods(bypassCache);
	} catch (err) {
		console.error(err);
	} finally {
		loading.value = false;
	}
}

/**
 * Shows the details dialog for a mod
 * @param {ResoluteMod} mod
 */
function showModDetails(mod) {
	router.replace(`${props.baseRoute}/${mod.id}`);
}

/**
 * Closes the mod details dialog if it is being shown
 */
function closeModDetails() {
	router.replace(props.baseRoute);
}

/**
 * Checks whether the Resonite path is configured and exists via the backend
 */
async function checkIfResonitePathExists() {
	if (!settings.current.resonitePath) {
		resonitePathExists.value = null;
	} else {
		resonitePathExists.value = await invoke('verify_resonite_path').catch(
			() => false,
		);
	}
}

let resizeInterval = null;
let resizingStartedAt = null;

/**
 * Begins an interval to resize the table based on the height of any alerts that may be showing.
 * The interval automatically cancels after 1 second since the most recent call to this function.
 * This is to work around an issue with table heights not necessarily fitting to parents perfectly.
 */
function adjustTableHeight() {
	if (!resizeInterval) {
		resizeInterval = setInterval(() => {
			alertHeight.value = alerts.value?.clientHeight ?? 0;
			if (Date.now() - resizingStartedAt > 1000) {
				clearInterval(resizeInterval);
				resizeInterval = null;
			}
		}, 100);
	}

	resizingStartedAt = Date.now();
}

/**
 * Handles the sidebar toggle event
 */
function onSidebarToggle() {
	adjustTableHeight();
}

const modTable = ref(null);
const expanded = ref(false);

/**
 * Expands or collapses all groups in the mod table
 */
function toggleAllGroups() {
	expanded.value = !expanded.value;
	if (expanded.value) modTable.value.expandAllGroups();
	else modTable.value.collapseAllGroups();
}
</script>
