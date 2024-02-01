<template>
	<v-dialog v-model="showDialog" persistent scrollable style="max-width: 960px">
		<v-card
			:title="`Resolute v${update?.version} is available!`"
			subtitle="Would you like to install the update now?"
		>
			<v-card-text>
				<div v-if="releaseNotes">
					<h3 class="text-h5">Release Notes</h3>
					<v-divider class="my-2" />
					<!-- eslint-disable vue/no-v-html -->
					<div
						class="pa-2 text-body-1 release-notes"
						v-html="releaseNotes"
					></div>
					<!-- eslint-enable vue/no-v-html -->
				</div>
			</v-card-text>

			<v-card-actions>
				<v-spacer />

				<v-btn
					:disabled="installingUpdate"
					variant="plain"
					@click="showDialog = false"
				>
					Not now
				</v-btn>
				<v-btn
					:loading="installingUpdate"
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
import { ref, onMounted } from 'vue';
import { check as tauriCheckUpdate } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { info, error } from '@tauri-apps/plugin-log';

import useNotifications from '../composables/notifications';
import { renderMarkdown } from '../util';

const notify = useNotifications();

const update = ref(null);
const releaseNotes = ref('');
const installingUpdate = ref(false);
const showDialog = ref(false);

onMounted(async () => {
	checkForUpdate();
});

/**
 * Checks for an app update and shows the update dialog if one is available
 */
async function checkForUpdate() {
	// Check for an update
	try {
		update.value = await tauriCheckUpdate();
		if (!update.value) return;

		info(`App update available (v${update.value.version})`);
		console.debug('App update', update.value);
	} catch (err) {
		error(`Error checking for app updates: ${err}`);
		return;
	}

	// Render the release notes
	try {
		releaseNotes.value = renderMarkdown(update.value.body);
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
		installingUpdate.value = true;
		await update.value.downloadAndInstall();
		await relaunch();
	} catch (err) {
		error(`Error installing app update: ${err}`);
		notify.error(
			'Error installing update',
			`Error installing app update:\n${err}`,
		);
	} finally {
		installingUpdate.value = false;
	}
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
