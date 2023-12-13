<template>
	<AppHeader title="All Mods">
		<template #actions>
			<v-tooltip text="Refresh mods" :open-delay="500">
				<template #activator="{ props }">
					<v-btn
						:icon="mdiRefresh"
						v-bind="props"
						@click="modStore.load(true)"
					/>
				</template>
			</v-tooltip>
		</template>
	</AppHeader>

	<v-main v-resize="adjustTableHeight">
		<v-alert
			v-if="!resonitePathExists"
			ref="resonitePathAlert"
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

		<ModTable
			:mods="modStore.mods"
			:disabled="!resonitePathExists"
			:style="`height: ${tableHeight}`"
		/>
	</v-main>
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
import { exists as fsExists } from '@tauri-apps/api/fs';
import { mdiRefresh } from '@mdi/js';

import useSettings from '../../settings';
import useModStore from '../../stores/mods';
import sidebarBus from '../../sidebar-bus';
import AppHeader from '../AppHeader.vue';
import ModTable from '../ModTable.vue';

const settings = useSettings();
const modStore = useModStore();
const resonitePathExists = ref(true);
const resonitePathAlert = ref(null);

const alertHeight = ref(0);
const tableHeight = computed(() => {
	if (!resonitePathAlert.value) return '100%';
	return `calc(100% - ${alertHeight.value}px)`;
});

onMounted(() => {
	if (!modStore.mods) modStore.load();
	sidebarBus.on('toggle', onSidebarToggle);
});

onUnmounted(() => {
	if (resizeInterval) {
		clearInterval(resizeInterval);
		resizeInterval = null;
	}

	sidebarBus.off('toggle', onSidebarToggle);
});

onBeforeMount(checkIfResonitePathExists);
watch(settings.current, checkIfResonitePathExists);

/**
 * Checks whether the Resonite path is configured and exists
 */
async function checkIfResonitePathExists() {
	if (!settings.current.resonitePath) {
		resonitePathExists.value = null;
	} else {
		resonitePathExists.value = await fsExists(
			settings.current.resonitePath,
		).catch(() => false);
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
			alertHeight.value = resonitePathAlert.value?.$el?.clientHeight ?? 0;
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
</script>
