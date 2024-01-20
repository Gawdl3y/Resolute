<template>
	<v-app :theme>
		<v-layout class="rounded rounded-md">
			<AppSidebar />
			<router-view />
		</v-layout>

		<UpdateDialog />
		<SetupGuideDialog v-if="!settings.current.setupGuideDone" />
	</v-app>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { info } from '@tauri-apps/plugin-log';

import useSettings from './composables/settings';
import AppSidebar from './components/AppSidebar.vue';
import UpdateDialog from './components/UpdateDialog.vue';
import SetupGuideDialog from './components/SetupGuideDialog.vue';

const settings = useSettings();
await settings.init();

const themeMediaMatcher = window.matchMedia('(prefers-color-scheme: dark)');
const systemTheme = ref(themeMediaMatcher.matches ? 'dark' : 'light');
const theme = computed(() => settings.current.theme ?? systemTheme.value);

onMounted(() => {
	info('App mounted - showing main window');
	setTimeout(() => invoke('show_window'), 50);
	themeMediaMatcher.addEventListener('change', onMatchMediaChange);
});

onUnmounted(() => {
	themeMediaMatcher.removeEventListener('change', onMatchMediaChange);
});

/**
 * Handles match media change events
 * @param {MediaQueryListEvent} evt
 */
function onMatchMediaChange(evt) {
	systemTheme.value = evt.matches ? 'dark' : 'light';
}
</script>
