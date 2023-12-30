<template>
	<v-dialog v-model="showDialog" persistent scrollable style="max-width: 960px">
		<v-card
			:title="`Resolute v${updateDetails?.version} is available!`"
			subtitle="Would you like to install the update now?"
		>
			<v-card-text>
				<div v-if="updateDetails?.notes">
					<h3 class="text-h5">Release Notes</h3>
					<v-divider class="my-2" />
					<!-- eslint-disable-next-line vue/no-v-html -->
					<div
						class="pa-2 text-body-1 release-notes"
						v-html="updateDetails.notes"
					></div>
				</div>
			</v-card-text>

			<v-card-actions>
				<v-spacer />

				<v-btn
					:disabled="updateDetails?.installing"
					variant="plain"
					@click="showDialog = false"
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
</template>

<script setup>
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import {
	checkUpdate as tauriCheckUpdate,
	installUpdate as tauriInstallUpdate,
	onUpdaterEvent as tauriOnUpdaterEvent,
} from '@tauri-apps/api/updater';
import { relaunch } from '@tauri-apps/api/process';
import { info, error } from 'tauri-plugin-log-api';
import { message } from '@tauri-apps/api/dialog';

import { renderMarkdown } from '../util';

const updateDetails = reactive({});
const showDialog = ref(false);
let unlistenToUpdaterEvents = null;

onMounted(async () => {
	unlistenToUpdaterEvents = await tauriOnUpdaterEvent(onUpdaterEvent);
	checkForUpdate();
});

onUnmounted(() => {
	unlistenToUpdaterEvents();
	unlistenToUpdaterEvents = null;
});

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
		updateDetails.notes = renderMarkdown(updateDetails.manifest.body);
	} catch (err) {
		error(`Error rendering app release notes: ${err}`);
	}

	showDialog.value = true;
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
		message(`Error installing app update:\n${err}`, {
			title: 'Error installing update',
			type: 'error',
		});
	} finally {
		updateDetails.installing = false;
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
	margin-bottom: 0.6em;
	font-size: 1.25rem;
	font-weight: normal;
}

.release-notes ul {
	margin-bottom: 1.5rem;
	padding-inline-start: 2em;
}
</style>
