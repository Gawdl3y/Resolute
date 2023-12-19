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
							<v-tooltip text="Select file" :open-delay="500">
								<template #activator="{ props: activator }">
									<FieldCopyButton :text="checksum" :hidden="checksumLoading" />
									<v-btn
										v-bind="activator"
										variant="text"
										:icon="mdiFileSearch"
										:loading="checksumLoading"
										@click="hashFile"
									/>
								</template>
							</v-tooltip>
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
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { open, message } from '@tauri-apps/api/dialog';
import { mdiFileCheck, mdiFileSearch } from '@mdi/js';

import AppHeader from '../AppHeader.vue';
import FieldCopyButton from '../FieldCopyButton.vue';

const checksum = ref('');
const checksumFile = ref('');
const checksumLoading = ref(false);

const fileHovering = ref(false);
let unlistenToFileDrop;
let unlistenToFileHover;
let unlistenToFileCancelled;

onMounted(async () => {
	unlistenToFileDrop = await listen('tauri://file-drop', (evt) => {
		console.debug('File drop received', evt);
		fileHovering.value = false;
		hashFile(evt.payload[0]);
	});

	unlistenToFileHover = await listen('tauri://file-drop-hover', (evt) => {
		console.debug('File hover received', evt);
		fileHovering.value = true;
	});

	unlistenToFileCancelled = await listen(
		'tauri://file-drop-cancelled',
		(evt) => {
			console.debug('File cancelled received', evt);
			fileHovering.value = false;
		},
	);
});

onUnmounted(() => {
	unlistenToFileDrop();
	unlistenToFileHover();
	unlistenToFileCancelled();
});

/**
 * Opens a dialog to choose a file, then requests the backend to calculate the checksum for that file
 */
async function hashFile(file = null) {
	// Prompt to choose a file
	if (!file) file = await open();
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
		message(`Error hashing file:\n${err}`, {
			title: 'Error hashing file',
			type: 'error',
		});
	} finally {
		checksumLoading.value = false;
	}
}
</script>
