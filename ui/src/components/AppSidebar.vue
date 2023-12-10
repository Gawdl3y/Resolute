<template>
	<el-menu
		router
		:default-active="router.currentRoute.value.path"
		:collapse="isCollapsed"
		class="sidebar-menu"
	>
		<el-menu-item-group>
			<el-menu-item index="/">
				<el-icon><i-ep-house /></el-icon>
				<template #title>Dashboard</template>
			</el-menu-item>

			<el-menu-item index="/mods">
				<el-icon><i-ep-goods /></el-icon>
				<template #title>All Mods</template>
			</el-menu-item>

			<el-menu-item index="/settings">
				<el-icon><i-ep-setting /></el-icon>
				<template #title>Settings</template>
			</el-menu-item>
		</el-menu-item-group>

		<el-menu-item @click="isCollapsed = !isCollapsed">
			<el-icon>
				<i-ep-expand v-if="isCollapsed" />
				<i-ep-fold v-else />
			</el-icon>
			<template #title>{{ isCollapsed ? 'Expand' : 'Collapse' }}</template>
		</el-menu-item>
	</el-menu>
</template>

<script setup>
import { ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();
const isCollapsed = ref(true);
</script>

<style>
.sidebar-menu:not(.el-menu--collapse) {
	width: 9em;
	min-height: 400px;
}
</style>
