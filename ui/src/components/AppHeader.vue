<template>
	<el-header height="64px">
		<el-page-header @back="goBack">
			<template #content>
				<el-space>
					<el-icon v-if="$slots.icon"><slot name="icon" /></el-icon>
					<el-text size="large"><slot /></el-text>
				</el-space>
			</template>
		</el-page-header>
	</el-header>
</template>

<script setup>
// import { computed } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();
// const canGoBack = computed(() => Boolean(window.history.state.back));

function goBack() {
	router.go(-1);
}
</script>

<style scoped>
.el-header {
	display: flex;
	align-items: center;
	border-bottom: 1px solid var(--el-menu-border-color);
}
</style>
