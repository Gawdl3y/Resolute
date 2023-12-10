<template>
	<el-container direction="vertical">
		<Header>
			<template #icon><i-ep-goods /></template>
			All Mods
		</Header>

		<el-main>
			<ModTable :mods="modStore.mods" />
		</el-main>
	</el-container>
</template>

<script setup>
import useModStore from '../../stores/mods';
import ModTable from '../ModTable.vue';

const modStore = useModStore();
const test = 42;
</script>
