<template>
	<el-container direction="vertical">
		<Header>
			<template #icon><i-ep-house /></template>
			Dashboard
		</Header>

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
</template>

<script setup>
import Header from '../Header.vue';
import LoadManifestButton from '../LoadManifestButton.vue';
</script>
