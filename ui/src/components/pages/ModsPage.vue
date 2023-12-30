<template>
	<AppHeader :title="title">
		<template #actions>
			<slot name="actions" />

			<v-tooltip text="Refresh mods" :open-delay="500">
				<template #activator="{ props: tooltipProps }">
					<v-btn
						:icon="mdiRefresh"
						:loading="loading"
						v-bind="tooltipProps"
						@click="loadModsFromFn(true)"
					/>
				</template>
			</v-tooltip>
		</template>
	</AppHeader>

	<v-main v-resize="adjustTableHeight">
		<div ref="alerts">
			<slot name="alerts" />
		</div>

		<ModTable
			:mods="mods"
			:disabled="disabled"
			:loading="loading"
			:style="`height: ${tableHeight}`"
			:allow-grouping="allowGrouping"
		/>
	</v-main>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { mdiRefresh } from '@mdi/js';

import sidebarBus from '../../sidebar-bus';
import AppHeader from '../AppHeader.vue';
import ModTable from '../ModTable.vue';

const props = defineProps({
	title: { type: String, required: true },
	mods: { type: Object, default: null },
	loadMods: { type: Function, required: true },
	disabled: { type: Boolean, default: false },
	allowGrouping: { type: Boolean, default: true },
});
const alerts = ref(null);
const loading = ref(false);

const alertHeight = ref(0);
const tableHeight = computed(() => {
	if (!alerts.value) return '100%';
	return `calc(100% - ${alertHeight.value}px)`;
});

onMounted(() => {
	if (!props.mods) loadModsFromFn(false);
	sidebarBus.on('toggle', onSidebarToggle);
});

onUnmounted(() => {
	if (resizeInterval) {
		clearInterval(resizeInterval);
		resizeInterval = null;
	}

	sidebarBus.off('toggle', onSidebarToggle);
});

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
</script>
