<template>
	<v-app :theme="theme">
		<v-layout class="rounded rounded-md">
			<AppSidebar />
			<router-view />
		</v-layout>
	</v-app>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import { info } from 'tauri-plugin-log-api';

import useSettings from './settings';
import AppSidebar from './components/AppSidebar.vue';

const settings = useSettings();
settings.init();

const themeMediaMatcher = window.matchMedia('(prefers-color-scheme: dark)');
const systemTheme = ref(themeMediaMatcher.matches ? 'dark' : 'light');
const theme = computed(() => settings.current.theme ?? systemTheme.value);

onMounted(() => {
	info('App mounted - showing main window');
	setTimeout(() => invoke('show_window'), 50);
	themeMediaMatcher.addEventListener('change', systemThemeListener);
});

onUnmounted(() => {
	themeMediaMatcher.removeEventListener('change', systemThemeListener);
});

function systemThemeListener(evt) {
	systemTheme.value = evt.matches ? 'dark' : 'light';
}
</script>
