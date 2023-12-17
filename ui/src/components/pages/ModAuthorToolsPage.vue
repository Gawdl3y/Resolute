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
						:loading="loading"
						readonly
					>
						<template #append-inner>
							<v-tooltip text="Select file" :open-delay="500">
								<template #activator="{ props: activator }">
									<FieldCopyButton :text="checksum" :hidden="loading" />
									<v-btn
										v-bind="activator"
										variant="text"
										:icon="mdiFileSearch"
										:loading="loading"
										@click="checksumFile"
									/>
								</template>
							</v-tooltip>
						</template>
					</v-text-field>
				</v-col>
			</v-row>
		</v-container>
	</v-main>
</template>

<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api';
import { open, message } from '@tauri-apps/api/dialog';
import { mdiFileSearch } from '@mdi/js';

import AppHeader from '../AppHeader.vue';
import FieldCopyButton from '../FieldCopyButton.vue';

const checksum = ref('');
const loading = ref(false);

/**
 * Opens a dialog to choose a file, then requests the backend to calculate the checksum for that file
 */
async function checksumFile() {
	// Prompt to choose a file
	const file = await open();
	if (!file) return;

	// Request the backend to checksum the selected file
	try {
		loading.value = true;
		checksum.value = 'Calculating...';
		checksum.value = await invoke('checksum_file', { file });
	} catch (err) {
		checksum.value = '';
		message(`Error hashing file:\n${err}`, {
			title: 'Error hashing file',
			type: 'error',
		});
	} finally {
		loading.value = false;
	}
}
</script>
