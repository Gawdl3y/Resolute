<template>
	<v-table>
		<thead>
			<tr>
				<th scope="col">Filename</th>
				<th scope="col">Destination</th>
				<th scope="col">URL</th>
				<th scope="col">SHA-256 Checksum</th>
			</tr>
		</thead>
		<tbody>
			<tr v-for="artifact of artifacts" :key="artifact.sha256">
				<td>
					<span v-if="artifact.filename">{{ artifact.filename }}</span>
					<SimpleTooltip
						v-else
						v-slot="{ props: tooltipProps }"
						text="Inferred from the URL"
					>
						<span v-bind="tooltipProps">
							{{ artifact.inferredFilename }}*
						</span>
					</SimpleTooltip>
				</td>
				<td>
					<span v-if="artifact.installLocation">
						{{ artifact.installLocation }}
					</span>
					<SimpleTooltip
						v-else
						v-slot="{ props: tooltipProps }"
						text="Default destination"
					>
						<span v-bind="tooltipProps">/rml_mods*</span>
					</SimpleTooltip>
				</td>
				<td>
					<a :href="artifact.url" target="_blank">{{ artifact.url }}</a>
				</td>
				<td>
					<TextCopier
						v-slot="{ props: copierProps, copy }"
						:text="artifact.sha256.toLowerCase()"
					>
						<ClickableSpan v-bind="copierProps" :action="copy">
							{{ artifact.sha256.toLowerCase() }}
						</ClickableSpan>
					</TextCopier>
				</td>
			</tr>
		</tbody>
	</v-table>
</template>

<script setup>
import TextCopier from '../TextCopier.vue';
import ClickableSpan from '../ClickableSpan.vue';
import SimpleTooltip from '../SimpleTooltip.vue';

defineProps({ artifacts: { type: Array, required: true } });
</script>
