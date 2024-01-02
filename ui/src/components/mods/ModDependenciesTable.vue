<template>
	<v-table>
		<thead>
			<th scope="col" class="text-start ps-4">Mod</th>
			<th scope="col" class="text-start ps-4">Version</th>
		</thead>
		<tbody>
			<tr v-for="depend of depends" :key="depend.mod">
				<td>{{ depend.mod }}</td>
				<td>{{ depend.semver }}</td>
			</tr>
		</tbody>
	</v-table>
</template>

<script setup>
import { computed } from 'vue';

import useModStore from '../../stores/mods';

const props = defineProps({ dependencies: { type: Object, required: true } });

const modStore = useModStore();
const depends = computed(() =>
	Object.entries(props.dependencies).map(([id, semver]) => {
		const mod = modStore?.mods[id];
		return {
			mod: mod ? `${mod.name} (${id})` : id,
			semver,
		};
	}),
);
</script>
