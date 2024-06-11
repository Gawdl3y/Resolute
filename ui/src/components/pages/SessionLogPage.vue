<template>
	<AppHeader title="Console" :loading :indeterminate="loading">
		<template #actions>
			<IconButton
				:icon="mdiFolderText"
				tooltip="Open log folder"
				@click="openLogs"
			/>
		</template>
	</AppHeader>

	<v-main>
		<v-sheet
			ref="sheet"
			class="h-100 w-100 d-flex flex-column-reverse pa-3 overflow-auto text-selectable"
		>
			<code>
				<!-- eslint-disable vue/no-v-html -->
				<pre v-html="log"></pre>
				<pre v-html="rtLog"></pre>
				<!-- eslint-enable vue/no-v-html -->
			</code>
		</v-sheet>
	</v-main>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { attachLogger } from '@tauri-apps/plugin-log';
import { mdiFolderText } from '@mdi/js';

import useNotifications from '../../composables/notifications';
import { escapeHTML } from '../../util';
import AppHeader from '../AppHeader.vue';
import IconButton from '../IconButton.vue';

const notify = useNotifications();

const sheet = ref(null);
const loading = ref(true);
const log = ref('');
const rtLog = ref('');

let detachLogger = null;

onMounted(async () => {
	// Add new log entries that come in
	detachLogger = await attachLogger((log) => {
		rtLog.value += `${rtLog.value ? '\n' : ''}${format(log.message)}`;
	});

	// Load the existing log entries from the backend
	try {
		log.value = format(await invoke('get_session_log'));
	} catch (err) {
		notify.error(
			'Error loading session log',
			`Error loading session log:\n${err}`,
		);
	} finally {
		loading.value = false;
	}
});

onUnmounted(() => {
	detachLogger();
});

/**
 * Formats a log message for "pretty" display via HTML
 * @param {string} text
 * @returns {string}
 */
function format(text) {
	return escapeHTML(text)
		.replace(
			/^(\[.+\])(\[.+\])(\[.+\])(\[.+\]) /gm,
			'<span class="font-weight-light"><span class="text-disabled">$1$2</span>$3<span class="text-primary">$4</span></span> ',
		)
		.replace(/\[ERROR\]/g, '<span class="text-error">[ERROR]</span>')
		.replace(/\[WARN\]/g, '<span class="text-warning">[WARN]</span>')
		.replace(/\[INFO\]/g, '<span class="text-secondary">[INFO]</span>')
		.replace(/\[(DEBUG|TRACE)\]/g, '<span class="text-disabled">[$1]</span>');
}

/**
 * Requests the backend to open the log directory in a file browser
 */
async function openLogs() {
	await invoke('open_log_dir');
}
</script>
