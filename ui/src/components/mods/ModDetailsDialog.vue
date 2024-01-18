<template>
	<v-dialog v-model="showDialog" scrollable style="max-width: 960px">
		<v-card :title="mod.name">
			<!-- Copiable mod ID subtitle -->
			<template #subtitle>
				<TextCopier v-slot="{ props: copierProps, copy }" :text="mod.id">
					<ClickableSpan v-bind="copierProps" :action="copy">
						{{ mod.id }}
					</ClickableSpan>
				</TextCopier>
			</template>

			<!-- Header actions and version selector -->
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

					<IconButton
						v-if="mod.website"
						:icon="mdiWeb"
						:href="mod.website"
						target="_blank"
						variant="text"
						tooltip="Website"
					/>

					<IconButton
						v-if="mod.sourceLocation"
						:icon="mdiSourceBranch"
						:href="mod.sourceLocation"
						target="_blank"
						variant="text"
						tooltip="Source"
					/>

					<IconButton
						:icon="mdiClose"
						variant="text"
						tooltip="Close"
						@click="close"
					/>
				</div>
			</template>

			<!-- Mod details body -->
			<v-card-text>
				<p class="text-body-1 mt-2 mb-6">{{ mod.description }}</p>

				<ModTags :mod class="mb-6" />

				<h2 class="text-h5 mb-2">Authors</h2>
				<ModAuthors :authors="mod.authors" class="mb-6" />

				<h2 class="d-flex align-center ga-2 text-h5 mb-2">
					Version {{ semver }}
					<IconButton
						v-if="version.releaseUrl"
						:icon="mdiLinkVariant"
						:href="version.releaseUrl"
						target="_blank"
						variant="text"
						density="comfortable"
						tooltip="Release page"
					/>
				</h2>
				<ModVersionInfoPanels :version />
			</v-card-text>

			<!-- Mod actions -->
			<v-card-actions>
				<v-spacer />

				<ModUninstaller
					v-if="mod.installedVersion"
					v-slot="{ uninstall, uninstalling, busy }"
					:mod
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
					:mod
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
					v-else-if="!version.isUnrecognized"
					v-slot="{ install, installing, busy }"
					:mod
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
import TextCopier from '../TextCopier.vue';
import ClickableSpan from '../ClickableSpan.vue';
import IconButton from '../IconButton.vue';

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

watch(showDialog, (show) => {
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
