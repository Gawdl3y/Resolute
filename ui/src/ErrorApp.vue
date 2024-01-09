<template>
	<v-app :theme="systemTheme">
		<v-main class="d-flex h-100 py-6 flex-column justify-center align-center">
			<v-card
				title="Initialization error"
				:prepend-icon="mdiAlert"
				style="max-width: 90vw"
			>
				<v-card-text>
					<p class="text-body-1">
						There was an error during Resolute's initialization. Additional
						information may be available in the logs. Consider reporting this
						problem by
						<a
							href="https://github.com/Gawdl3y/Resolute/issues/new"
							target="_blank"
							>opening an issue</a
						>
						on the GitHub repository.
					</p>

					<h2 class="text-h6 mt-6 mb-2">Error details</h2>
					<v-textarea
						v-model="error"
						variant="solo-filled"
						readonly
						no-resize
						hide-details
						:rows="errorRows"
						class="text-mono"
					>
						<template #append-inner>
							<CopyButton :text="error" />
						</template>
					</v-textarea>
				</v-card-text>

				<v-card-actions>
					<v-btn @click="relaunch">Relaunch</v-btn>
					<v-btn @click="openLogs">View logs</v-btn>
				</v-card-actions>
			</v-card>
		</v-main>
	</v-app>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import { relaunch } from '@tauri-apps/api/process';
import { info } from 'tauri-plugin-log-api';
import { mdiAlert } from '@mdi/js';

import CopyButton from './components/CopyButton.vue';

const themeMediaMatcher = window.matchMedia('(prefers-color-scheme: dark)');
const systemTheme = ref(themeMediaMatcher.matches ? 'dark' : 'light');

const error = globalThis.error;
const errorRows = Math.min(Math.max(error.match(/\n/g).length + 2, 5), 10);

onMounted(() => {
	info('ErrorApp mounted - showing error window');
	setTimeout(() => invoke('show_window'), 50);
	themeMediaMatcher.addEventListener('change', onMatchMediaChange);
});

onUnmounted(() => {
	themeMediaMatcher.removeEventListener('change', onMatchMediaChange);
});

/**
 * Requests the backend to open the log directory in a file browser
 */
async function openLogs() {
	await invoke('open_log_dir');
}

/**
 * Handles match media change events
 * @param {MediaQueryListEvent} evt
 */
function onMatchMediaChange(evt) {
	systemTheme.value = evt.matches ? 'dark' : 'light';
}
</script>
