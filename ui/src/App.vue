<template>
	<v-app :theme="theme">
		<v-layout class="rounded rounded-md">
			<AppSidebar />
			<router-view />
		</v-layout>
	</v-app>
</template>

<script setup>
import { computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import { info } from 'tauri-plugin-log-api';

import useSettings from './settings';
import AppSidebar from './components/AppSidebar.vue';

const settings = useSettings();
settings.init();

const theme = computed(
	() =>
		settings.current.theme ??
		(window.matchMedia('(prefers-color-scheme: dark)') ? 'dark' : 'light'),
);

onMounted(() => {
	info('App mounted - showing main window');
	setTimeout(() => invoke('show_window'), 50);
});
</script>
