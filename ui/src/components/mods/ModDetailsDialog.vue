<template>
	<v-dialog v-model="showDialog" scrollable style="max-width: 960px">
		<v-card :title="mod.name" :subtitle="mod.id">
			<v-card-text>
				<p class="text-body-1 mt-2 mb-6">{{ mod.description }}</p>

				<ModTags :mod="mod" class="mb-6" />

				<h2 class="text-h5 mb-2">Authors</h2>
				<ModAuthors :authors="mod.authors" class="mb-6" />

				<h2 class="text-h5 mb-2">Version v{{ semver }}</h2>
				<ModVersionInfoPanels :version="version" />
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
				<div class="d-flex">
					<v-select
						v-model="semver"
						label="Version"
						:items="versions"
						item-title="semver"
						item-value="semver"
						variant="solo-filled"
						density="comfortable"
						hide-details
						class="me-4"
					/>

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
				</div>
			</template>
		</v-card>
	</v-dialog>
</template>

<script setup>
import { ref, computed, watch } from 'vue';
import {
	mdiDownload,
	mdiDelete,
	mdiUpdate,
	mdiRefresh,
	mdiClose,
	mdiWeb,
	mdiSourceBranch,
} from '@mdi/js';

import ModInstaller from './ModInstaller.vue';
import ModUninstaller from './ModUninstaller.vue';
import ModUpdater from './ModUpdater.vue';
import ModTags from './ModTags.vue';
import ModAuthors from './ModAuthors.vue';
import ModVersionInfoPanels from './ModVersionInfoPanels.vue';

const props = defineProps({
	mod: { type: Object, required: true },
	disabled: { type: Boolean, default: false },
});
const emit = defineEmits(['close']);

const showDialog = ref(true);
const versions = computed(() => Object.values(props.mod.versions));
const version = computed(() => props.mod.versions[semver.value]);
const semver = ref(
	(props.mod.installedVersion ?? props.mod.latestVersion).semver,
);

watch(showDialog, (_, show) => {
	if (!show) emit('close');
});

watch(
	() => props.mod,
	() => {
		semver.value = (
			props.mod.installedVersion ?? props.mod.latestVersion
		).semver;
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
