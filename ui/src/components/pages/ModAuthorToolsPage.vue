<template>
	<AppHeader title="Mod Authoring Tools" />

	<v-main>
		<v-container>
			<v-row>
				<v-col>
					<v-text-field
						v-model="checksum"
						variant="solo"
						label="SHA-256 Checksum"
						:hint="checksumFile"
						:persistent-hint="Boolean(checksumFile)"
						:loading="checksumLoading"
						readonly
					>
						<template #append-inner>
							<CopyButton :text="checksum" :hidden="checksumLoading" />
							<IconButton
								:icon="mdiFileSearch"
								:loading="checksumLoading"
								tooltip="Select file"
								variant="text"
								@click="hashFile()"
							/>
						</template>
					</v-text-field>
				</v-col>
			</v-row>
		</v-container>
	</v-main>

	<v-overlay v-model="fileHovering" class="align-center justify-center">
		<v-card class="d-flex align-center justify-center ga-4 pa-3 text-h4">
			<v-icon :icon="mdiFileCheck" />
			Calculate checksum
		</v-card>
	</v-overlay>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { mdiFileCheck, mdiFileSearch } from '@mdi/js';

import useNotifications from '../../composables/notifications';
import AppHeader from '../AppHeader.vue';
import CopyButton from '../CopyButton.vue';
import IconButton from '../IconButton.vue';

const notify = useNotifications();

const checksum = ref('');
const checksumFile = ref('');
const checksumLoading = ref(false);

const fileHovering = ref(false);
let unlistenToDragDrop;
let unlistenToDragHover;
let unlistenToDragCancelled;

onMounted(async () => {
	unlistenToDragDrop = await listen('tauri://drop', (evt) => {
		console.debug('File drop received', evt);
		fileHovering.value = false;
		hashFile(evt.payload.paths[0]);
	});

	unlistenToDragHover = await listen('tauri://drop-over', (evt) => {
		console.debug('File hover received', evt);
		if (evt.payload) fileHovering.value = true;
	});

	unlistenToDragCancelled = await listen('tauri://drag-cancelled', (evt) => {
		console.debug('File cancelled received', evt);
		fileHovering.value = false;
	});
});

onUnmounted(() => {
	unlistenToDragDrop();
	unlistenToDragHover();
	unlistenToDragCancelled();
});

/**
 * Opens a dialog to choose a file, then requests the backend to calculate the checksum for that file
 * @param {string} [file=null] File path to hash (skips dialog if provided)
 */
async function hashFile(file = null) {
	// Prompt to choose a file
	if (!file) file = (await open())?.path;
	if (!file) return;

	// Request the backend to checksum the selected file
	try {
		checksumLoading.value = true;
		checksumFile.value = file;
		checksum.value = 'Calculating...';
		checksum.value = await invoke('hash_file', { path: file });
	} catch (err) {
		checksumFile.value = '';
		checksum.value = '';
		notify.error('Error hashing file', `Error hashing file:\n${err}`);
	} finally {
		checksumLoading.value = false;
	}
}
</script>
