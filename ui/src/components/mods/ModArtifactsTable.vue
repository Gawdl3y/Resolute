<template>
	<v-table>
		<thead>
			<th scope="col">Filename</th>
			<th scope="col">Destination</th>
			<th scope="col">URL</th>
			<th scope="col">SHA-256 Checksum</th>
		</thead>
		<tbody>
			<tr v-for="artifact of artifacts" :key="artifact.sha256">
				<td>
					<span v-if="artifact.filename">{{ artifact.filename }}</span>
					<v-tooltip v-else text="Inferred from the URL">
						<template #activator="{ props: tooltipProps }">
							<span v-bind="tooltipProps">
								{{ artifact.inferredFilename }}*
							</span>
						</template>
					</v-tooltip>
				</td>
				<td>
					<span v-if="artifact.installLocation">
						{{ artifact.installLocation }}
					</span>
					<v-tooltip v-else text="Default destination">
						<template #activator="{ props: tooltipProps }">
							<span v-bind="tooltipProps">/rml_mods*</span>
						</template>
					</v-tooltip>
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
