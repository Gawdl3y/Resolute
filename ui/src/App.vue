<template>
	<v-app :theme="theme">
		<v-layout class="rounded rounded-md">
			<AppSidebar />
			<router-view />
		</v-layout>

		<v-dialog
			v-model="updateDialog"
			persistent
			scrollable
			style="max-width: 960px"
		>
			<v-card
				:title="`Resolute v${updateDetails?.version} is available!`"
				subtitle="Would you like to install the update now?"
			>
				<v-card-text>
					<div v-if="updateDetails?.notes">
						<h3 class="text-h5">Release Notes</h3>
						<v-divider class="mt-2" />
						<!-- eslint-disable-next-line vue/no-v-html -->
						<div class="release-notes pa-2" v-html="updateDetails.notes"></div>
					</div>
				</v-card-text>

				<v-card-actions>
					<v-spacer />

					<v-btn
						:disabled="updateDetails?.installing"
						variant="plain"
						@click="updateDialog = false"
					>
						Not now
					</v-btn>
					<v-btn
						:loading="updateDetails?.installing"
						class="text-primary font-weight-bold"
						@click="installUpdate"
					>
						Update
					</v-btn>
				</v-card-actions>
			</v-card>
		</v-dialog>
	</v-app>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import {
	checkUpdate as tauriCheckUpdate,
	installUpdate as tauriInstallUpdate,
	onUpdaterEvent as tauriOnUpdaterEvent,
} from '@tauri-apps/api/updater';
import { relaunch } from '@tauri-apps/api/process';
import { info, error } from 'tauri-plugin-log-api';

import useSettings from './settings';
import { renderReleaseNotes } from './util';
import AppSidebar from './components/AppSidebar.vue';

const settings = useSettings();
settings.init();

const themeMediaMatcher = window.matchMedia('(prefers-color-scheme: dark)');
const systemTheme = ref(themeMediaMatcher.matches ? 'dark' : 'light');
const theme = computed(() => settings.current.theme ?? systemTheme.value);

const updateDialog = ref(false);
const updateDetails = reactive({});
let unlistenToUpdaterEvents = null;

onMounted(async () => {
	info('App mounted - showing main window');
	setTimeout(() => invoke('show_window'), 50);

	themeMediaMatcher.addEventListener('change', onMatchMediaChange);

	unlistenToUpdaterEvents = await tauriOnUpdaterEvent(onUpdaterEvent);
	checkForUpdate();
});

onUnmounted(() => {
	themeMediaMatcher.removeEventListener('change', onMatchMediaChange);

	unlistenToUpdaterEvents();
	unlistenToUpdaterEvents = null;
});

/**
 * Handles match media change events
 * @param {*} evt
 */
function onMatchMediaChange(evt) {
	systemTheme.value = evt.matches ? 'dark' : 'light';
}

/**
 * Checks for an app update and shows the update dialog if one is available
 */
async function checkForUpdate() {
	// Check for an update
	try {
		const { shouldUpdate, manifest } = await tauriCheckUpdate();
		if (!shouldUpdate) return;

		info(`App update available (v${manifest.version})`);
		console.debug('App update manifest', manifest);
		updateDetails.manifest = manifest;
		updateDetails.version = manifest.version;
	} catch (err) {
		error(`Error checking for app updates: ${err}`);
		return;
	}

	// Render the release notes
	try {
		updateDetails.notes = renderReleaseNotes(updateDetails.manifest.body);
	} catch (err) {
		error(`Error rendering app release notes: ${err}`);
	}

	updateDialog.value = true;
}

/**
 * Installs the available app update
 */
async function installUpdate() {
	try {
		updateDetails.installing = true;
		await tauriInstallUpdate();
		await relaunch();
	} catch (err) {
		error(`Error installing app update: ${err}`);
	}
}

/**
 * Handles app updater events
 * @param {Object} data
 */
function onUpdaterEvent({ error, status }) {
	if (error) error(`App updater error received (${status}): ${error}`);
	else info(`App updater event received: ${status}`);
}
</script>

<style>
.release-notes h2 {
	margin-top: 1em;
	margin-bottom: 0.6em;
	font-size: 1.25em;
	font-weight: normal;
}

.release-notes ul {
	margin-bottom: 0.5em;
	padding-inline-start: 2em;
}
</style>
