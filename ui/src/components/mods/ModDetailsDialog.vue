<template>
	<v-dialog v-model="showDialog" scrollable style="max-width: 960px">
		<v-card
			:title="mod.name"
			:subtitle="`v${(mod.installedVersion ?? mod.latestVersion).semver}`"
		>
			<v-card-text>
				<p class="text-body-1 mb-6">
					{{ mod.description }}
				</p>

				<div class="d-flex flex-wrap ga-2 mb-6">
					<v-chip color="primary">{{ mod.category }}</v-chip>
					<v-chip
						v-for="flag of mod.flags"
						:key="flag"
						:color="
							flag === 'deprecated'
								? 'error'
								: flag === 'final'
									? 'warning'
									: 'secondary'
						"
					>
						{{ flag[0].toUpperCase() }}{{ flag.substring(1) }}
					</v-chip>
					<v-chip v-for="tag of mod.tags" :key="tag">{{ tag }}</v-chip>
				</div>

				<h2 class="text-h5 mb-2">Authors</h2>

				<div class="d-flex flex-wrap ga-4 mb-8">
					<v-card
						v-for="author of mod.authors"
						:key="author.name"
						:title="author.name"
						:href="author.url"
						target="_blank"
						variant="tonal"
						density="comfortable"
					>
						<template #prepend>
							<v-avatar v-if="author.icon" :image="author.icon" size="large" />
							<v-avatar
								v-else
								:icon="mdiAccount"
								size="large"
								color="surface-variant"
							/>
						</template>

						<template #append>
							<v-tooltip
								v-if="author.support"
								:text="`Support ${author.name}`"
								:open-delay="500"
							>
								<template #activator="{ props: tooltipProps }">
									<v-btn
										v-bind="tooltipProps"
										:icon="mdiGift"
										:href="author.support"
										target="_blank"
										variant="text"
									/>
								</template>
							</v-tooltip>
						</template>
					</v-card>
				</div>

				<v-expansion-panels class="mod-details-expansion">
					<v-expansion-panel bg-color="rgba(var(--v-theme-on-surface), 0.12)">
						<v-expansion-panel-title>
							<h2 class="text-h6">Artifacts</h2>
						</v-expansion-panel-title>

						<v-expansion-panel-text>
							<v-table>
								<thead>
									<th scope="col">Filename</th>
									<th scope="col">Destination</th>
									<th scope="col">URL</th>
									<th scope="col">Checksum</th>
								</thead>
								<tbody>
									<tr
										v-for="artifact of (
											mod.installedVersion ?? mod.latestVersion
										).artifacts"
										:key="artifact.sha256"
									>
										<td>
											<span v-if="artifact.filename">{{
												artifact.filename
											}}</span>
											<span v-else class="text-disabled"
												>&lt;unspecified&gt;</span
											>
										</td>
										<td>
											<span v-if="artifact.installLocation">{{
												artifact.installLocation
											}}</span>
											<span v-else class="text-disabled"
												>&lt;unspecified&gt;</span
											>
										</td>
										<td>
											<a :href="artifact.url" target="_blank">{{
												artifact.url
											}}</a>
										</td>
										<td>{{ artifact.sha256.toLowerCase() }}</td>
									</tr>
								</tbody>
							</v-table>
						</v-expansion-panel-text>
					</v-expansion-panel>
				</v-expansion-panels>
			</v-card-text>

			<v-card-actions>
				<v-spacer />

				<ModUninstaller
					v-if="mod.installedVersion"
					v-slot="{ uninstall, uninstalling, busy }"
					:mod="mod"
				>
					<v-btn
						:prepend-icon="mdiDelete"
						:disabled="disabled || (busy && !uninstalling)"
						:loading="uninstalling"
						@click="uninstall"
					>
						Uninstall
					</v-btn>
				</ModUninstaller>

				<ModInstaller
					v-if="!mod.hasUpdate"
					v-slot="{ install, installing, busy }"
					:mod="mod"
				>
					<v-btn
						:prepend-icon="mod.installedVersion ? mdiRefresh : mdiDownload"
						:disabled="disabled || (busy && !installing)"
						:loading="installing"
						@click="install"
					>
						{{ mod.installedVersion ? 'Reinstall' : 'Install' }}
					</v-btn>
				</ModInstaller>

				<ModUpdater v-else v-slot="{ update, updating, busy }" :mod="mod">
					<v-btn
						:prepend-icon="mdiUpdate"
						:disabled="disabled || (busy && !updating)"
						:loading="updating"
						@click="update"
					>
						Update
					</v-btn>
				</ModUpdater>
			</v-card-actions>

			<template #append>
				<v-tooltip v-if="mod.website" text="Website" :open-delay="500">
					<template #activator="{ props: tooltipProps }">
						<v-btn
							v-bind="tooltipProps"
							:icon="mdiWeb"
							:href="mod.website"
							target="_blank"
							variant="text"
						/>
					</template>
				</v-tooltip>

				<v-tooltip v-if="mod.sourceLocation" text="Source" :open-delay="500">
					<template #activator="{ props: tooltipProps }">
						<v-btn
							v-bind="tooltipProps"
							:icon="mdiSourceBranch"
							:href="mod.sourceLocation"
							target="_blank"
							variant="text"
						/>
					</template>
				</v-tooltip>

				<v-tooltip text="Close" :open-delay="500">
					<template #activator="{ props: tooltipProps }">
						<v-btn
							v-bind="tooltipProps"
							:icon="mdiClose"
							variant="text"
							@click="close"
						/>
					</template>
				</v-tooltip>
			</template>
		</v-card>
	</v-dialog>
</template>

<script setup>
import { ref, watch } from 'vue';
import {
	mdiDownload,
	mdiDelete,
	mdiUpdate,
	mdiRefresh,
	mdiAccount,
	mdiGift,
	mdiClose,
	mdiWeb,
	mdiSourceBranch,
} from '@mdi/js';

import ModInstaller from './ModInstaller.vue';
import ModUninstaller from './ModUninstaller.vue';
import ModUpdater from './ModUpdater.vue';

const props = defineProps({
	mod: { type: Object, required: true },
	disabled: { type: Boolean, default: false },
});
const emit = defineEmits(['close']);

const showDialog = ref(true);

watch(showDialog, (_, show) => {
	if (!show) emit('close');
});

watch(
	() => props.mod,
	() => {
		showDialog.value = true;
	},
);

/**
 * Closes the dialog
 */
function close() {
	showDialog.value = false;
}
</script>

<style>
.mod-details-expansion .v-expansion-panel-text__wrapper {
	padding: 0 !important;
}

.mod-details-expansion .v-table {
	padding: 1em 0 0 0;
	background: rgba(var(--v-theme-surface), var(--v-medium-emphasis-opacity));
}
</style>
