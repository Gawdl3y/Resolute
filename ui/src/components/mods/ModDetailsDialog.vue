<template>
	<v-dialog v-model="showDialog" scrollable style="max-width: 960px">
		<v-card :title="mod.name" :subtitle="mod.id">
			<v-card-text>
				<p class="text-body-1 mt-2 mb-6">{{ mod.description }}</p>

				<ModTags :mod="mod" class="mb-6" />

				<h2 class="text-h5 mb-2">Authors</h2>
				<ModAuthors :authors="mod.authors" class="mb-6" />

				<h2 class="d-flex align-center ga-2 text-h5 mb-2">
					Version v{{ semver }}
					<v-btn
						v-if="version.releaseUrl"
						:icon="mdiLinkVariant"
						:href="version.releaseUrl"
						target="_blank"
						variant="text"
						density="comfortable"
					/>
				</h2>
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

				<ModUpdater
					v-if="updateAvailable"
					v-slot="{ update, updating, busy }"
					:mod="mod"
					:version="semver"
				>
					<v-btn
						:prepend-icon="mdiUpdate"
						:disabled="disabled || (busy && !updating)"
						:loading="updating"
						@click="update"
					>
						{{ updateText }}
					</v-btn>
				</ModUpdater>

				<ModInstaller
					v-else
					v-slot="{ install, installing, busy }"
					:mod="mod"
					:version="semver"
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
			</v-card-actions>

			<template #append>
				<div class="d-flex">
					<v-select
						v-model="semver"
						label="Version"
						:items="versions"
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
import { lte as semverLt } from 'semver';
import {
	mdiDownload,
	mdiDelete,
	mdiUpdate,
	mdiRefresh,
	mdiClose,
	mdiWeb,
	mdiSourceBranch,
	mdiLinkVariant,
	mdiCheck,
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
const versions = computed(() =>
	Object.keys(props.mod.versions).map((ver) => ({
		title: ver,
		value: ver,
		props:
			ver === props.mod.installedVersion?.semver
				? { appendIcon: mdiCheck }
				: undefined,
	})),
);
const version = computed(() => props.mod.versions[semver.value]);
const semver = ref(props.mod.latestVersion.semver);
const updateAvailable = computed(() => {
	if (!props.mod.installedVersion) return false;
	if (props.mod.installedVersion.semver === semver.value) return false;
	return true;
});
const updateText = computed(() => {
	if (!props.mod.installedVersion) return null;
	if (semverLt(semver.value, props.mod.installedVersion.semver)) {
		return `Downgrade to ${semver.value}`;
	}
	return `Update to ${semver.value}`;
});

watch(showDialog, (_, show) => {
	if (!show) emit('close');
});

watch(
	() => props.mod,
	() => {
		semver.value = props.mod.latestVersion.semver;
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
