<template>
	<el-container>
		<el-aside width="100px">
			Aside
		</el-aside>

		<el-container>
			<el-header>
				Header
			</el-header>

			<el-main>
				<LoadManifestButton type="primary">
					<template #icon><i-ep-refresh /></template>
					Load manifest
				</LoadManifestButton>
				<LoadManifestButton bypass-cache>
					<template #icon><i-ep-download /></template>
					Force download manifest
				</LoadManifestButton>
			</el-main>
		</el-container>
	</el-container>
</template>

<script setup>
import { onMounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import { info } from 'tauri-plugin-log-api';
import LoadManifestButton from './components/LoadManifestButton.vue';

onMounted(() => {
	info('App mounted - showing main window');
	setTimeout(() => invoke('show_window'), 50);
});
</script>
