<template>
	<v-table>
		<thead>
			<tr>
				<th scope="col" class="text-start ps-4">Mod</th>
				<th scope="col" class="text-start ps-4">Version</th>
			</tr>
		</thead>
		<tbody>
			<tr v-for="depend of depends" :key="depend.id">
				<td>
					<span v-if="depend.name">
						{{ depend.name }}
						<span class="text-disabled">({{ depend.id }})</span>
					</span>
					<span v-else>{{ depend.id }}</span>
				</td>
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
			name: mod ? mod.name : null,
			id,
			semver,
		};
	}),
);
</script>
