<template>
	<AppHeader title="Console" :loading="loading" :indeterminate="loading">
		<template #actions>
			<SimpleTooltip v-slot="{ props }" text="Open log folder">
				<v-btn v-bind="props" :icon="mdiFolderText" @click="openLogs" />
			</SimpleTooltip>
		</template>
	</AppHeader>

	<v-main>
		<v-sheet
			ref="sheet"
			class="h-100 w-100 d-flex flex-column-reverse pa-3 overflow-auto text-selectable"
		>
			<code>
				<!-- eslint-disable-next-line vue/no-v-html -->
				<pre v-html="log + rtLog"></pre>
			</code>
		</v-sheet>
	</v-main>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { message } from '@tauri-apps/api/dialog';
import { mdiFolderText } from '@mdi/js';

import { escapeHTML } from '../../util';
import AppHeader from '../AppHeader.vue';
import SimpleTooltip from '../SimpleTooltip.vue';

const sheet = ref(null);
const loading = ref(true);
const log = ref('');
const rtLog = ref('');

let unlistenToLogEvents = null;

onMounted(async () => {
	// Listen for log events
	unlistenToLogEvents = await listen('log://log', onLogEvent);

	// Load the existing log entries from the backend
	try {
		log.value = format(await invoke('get_session_log'));
	} catch (err) {
		message(`Error loading session log:\n${err}`, {
			title: 'Error loading session log',
			type: 'error',
		});
	} finally {
		loading.value = false;
	}
});

onUnmounted(() => {
	unlistenToLogEvents();
});

/**
 * Formats a log message for "pretty" display via HTML
 * @param {string} text
 * @returns {string}
 */
function format(text) {
	return escapeHTML(text)
		.replace(
			/^(\[.+\])(\[.+\])(\[.+\])(\[.+\])\s/gm,
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

/**
 * Handles a log event
 * @param {Object} evt
 */
function onLogEvent(evt) {
	// Strip ANSI control codes
	// https://github.com/tauri-apps/tauri-plugin-log/blob/19f5dcc0425e9127d2c591780e5047b83e77a7c2/guest-js/index.ts#L195C31-L195C31
	const message = evt.payload.message.replace(
		// eslint-disable-next-line no-control-regex
		/[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g,
		'',
	);

	rtLog.value += `${rtLog.value ? '\n' : ''}${format(message)}`;
}
</script>
