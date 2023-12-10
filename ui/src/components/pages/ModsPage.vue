<template>
	<el-container direction="vertical">
		<AppHeader>
			<template #icon><i-ep-goods /></template>
			All Mods
		</AppHeader>

		<el-main>
			<ModTable :mods="modStore.mods" />
		</el-main>
	</el-container>
</template>

<script setup>
import useModStore from '../../stores/mods';
import AppHeader from '../AppHeader.vue';
import ModTable from '../ModTable.vue';

const modStore = useModStore();
</script>
