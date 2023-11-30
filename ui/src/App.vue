<template>
	<el-container id="main">
		<Sidebar />
		<router-view />
	</el-container>
</template>

<script setup>
import { onMounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import { info } from 'tauri-plugin-log-api';

import useSettings from './settings';
import Sidebar from './components/Sidebar.vue';

const settings = useSettings();
settings.init();

onMounted(() => {
	info('App mounted - showing main window');
	setTimeout(() => invoke('show_window'), 50);
});
</script>

<style scoped>
#main {
	height: 100%;
}
</style>
