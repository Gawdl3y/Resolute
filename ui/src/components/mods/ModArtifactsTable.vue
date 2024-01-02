<template>
	<v-table>
		<thead>
			<th scope="col">Filename</th>
			<th scope="col">Destination</th>
			<th scope="col">URL</th>
			<th scope="col">Checksum</th>
		</thead>
		<tbody>
			<tr v-for="artifact of artifacts" :key="artifact.sha256">
				<td>
					<span v-if="artifact.filename">{{ artifact.filename }}</span>
					<span v-else class="text-disabled">&lt;unspecified&gt;</span>
				</td>
				<td>
					<span v-if="artifact.installLocation">
						{{ artifact.installLocation }}
					</span>
					<span v-else class="text-disabled">&lt;unspecified&gt;</span>
				</td>
				<td>
					<a :href="artifact.url" target="_blank">{{ artifact.url }}</a>
				</td>
				<td>{{ artifact.sha256.toLowerCase() }}</td>
			</tr>
		</tbody>
	</v-table>
</template>

<script setup>
defineProps({ artifacts: { type: Array, required: true } });
</script>
