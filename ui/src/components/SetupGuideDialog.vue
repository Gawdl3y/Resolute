<template>
	<v-dialog
		v-model="showDialog"
		:persistent="!settings.current.allowClosingSetupGuide"
		style="max-width: 960px"
		@update:model-value="onModelChange"
	>
		<v-card>
			<v-stepper v-model="step">
				<v-stepper-header>
					<v-stepper-item value="1">Welcome</v-stepper-item>
					<v-divider />
					<v-stepper-item
						value="2"
						:complete="resonitePathSelected"
						:error="!resonitePathSelected && step > 1"
					>
						Resonite path
					</v-stepper-item>
					<v-divider />
					<v-stepper-item
						value="3"
						:complete="prereqsInstalled >= 2"
						:error="prereqsInstalled < 2 && step > 2"
					>
						Prerequisites
					</v-stepper-item>
					<v-divider />
					<v-stepper-item value="4">Steam launch option</v-stepper-item>
				</v-stepper-header>

				<v-stepper-window v-model="step">
					<!-- Welcome page -->
					<v-stepper-window-item value="1">
						<h2 class="text-h4 mb-3">Welcome!</h2>
						<p class="mb-3 text-body-1">
							Thanks for giving Resolute a try!<br />Remember that it's very
							much a work in progress, although the goal is definitely to make
							your life easier.
						</p>
						<p class="text-body-1">
							Modding Resonite is easy and can really enhance your experience!
							There are just a few simple steps to complete before getting
							started. If you already know what you're doing and have everything
							set up, feel free to skip through this process.
						</p>
					</v-stepper-window-item>

					<!-- Resonite path selection page -->
					<v-stepper-window-item value="2">
						<h2 class="text-h4 mb-3">Resonite path</h2>
						<p class="mb-3 text-body-1">
							In order to install mods for you, Resolute needs to know where
							Resonite is installed on your computer.
						</p>
						<p class="mb-3 text-body-1">
							Resolute attempts to find this automatically, but it may not be
							able to if you've put Resonite in a secondary Steam library, such
							as on a different drive.
						</p>
						<p class="mb-5 text-body-1">
							<strong>Select your Resonite install folder below.</strong>
						</p>

						<ResonitePathSetting variant="solo-filled" />
					</v-stepper-window-item>

					<!-- Prerequisite mod installation page -->
					<v-stepper-window-item value="3">
						<h2 class="text-h4 mb-3">Prerequisites</h2>
						<p class="mb-4 text-body-1">
							The first thing we need to set up is the mod loader. There are two
							main components for it.<br />
							<strong>Click the below buttons to install them.</strong>
						</p>

						<p class="mb-1 text-body-1">
							<a
								href="https://github.com/resonite-modding-group/ResoniteModLoader"
								target="_blank"
								>ResoniteModLoader</a
							>
							(RML) is the plugin responsible for loading all mods.
						</p>
						<ModInstaller
							v-slot="{ install, busy, installing, installed }"
							mod="com.resonitemodloader.ResoniteModLoader"
							@install="prereqsInstalled++"
						>
							<v-btn
								variant="tonal"
								:prepend-icon="mdiDownload"
								:disabled="
									installed ||
									!modStore.mods ||
									!resonitePathSelected ||
									(busy && !installing)
								"
								:loading="installing"
								class="ms-4 mt-2"
								@click="install"
							>
								{{ installed ? 'Installed' : 'Install' }} RML
							</v-btn>
						</ModInstaller>

						<p class="mt-6 mb-1 text-body-1">
							<a href="https://harmony.pardeike.net/" target="_blank"
								>Harmony</a
							>
							is a library that allows RML and mods to interface directly with
							Resonite code and modify it at runtime.
						</p>
						<ModInstaller
							v-slot="{ install, busy, installing, installed }"
							mod="net.pardeike.harmony"
							@install="prereqsInstalled++"
						>
							<v-btn
								variant="tonal"
								:prepend-icon="mdiDownload"
								:disabled="
									installed ||
									!modStore.mods ||
									!resonitePathSelected ||
									(busy && !installing)
								"
								:loading="installing"
								class="ms-4 mt-2"
								@click="install"
							>
								{{ installed ? 'Installed' : 'Install' }} Harmony
							</v-btn>
						</ModInstaller>
					</v-stepper-window-item>

					<!-- Steam launch option page -->
					<v-stepper-window-item value="4">
						<h2 class="text-h4 mb-3">Steam launch option</h2>
						<p class="mb-3 text-body-1">
							In order for RML to actually load mods, you need to tell Resonite
							to load RML itself!
						</p>
						<p class="mb-5 text-body-1">
							In Steam, right-click Resonite in your library and choose
							<strong>Properties</strong>.<br />In that menu, under the
							<strong>General</strong> tab, enter the following into the text
							field under <strong>Launch options</strong>:
						</p>

						<v-text-field
							model-value="-LoadAssembly Libraries/ResoniteModLoader.dll"
							variant="solo-filled"
							readonly
							hide-details
							class="mb-5"
						>
							<template #append-inner>
								<FieldCopyButton
									text="-LoadAssembly Libraries/ResoniteModLoader.dll"
								/>
							</template>
						</v-text-field>

						<p class="text-body-1">
							For additional guidance, see the
							<a
								href="https://github.com/resonite-modding-group/ResoniteModLoader/wiki/Launch-Options"
								target="_blank"
								>RML documentation</a
							>.
						</p>
					</v-stepper-window-item>
				</v-stepper-window>

				<v-stepper-actions
					:disabled="step === 0 ? 'prev' : false"
					:next-text="step < 3 ? 'Next' : 'Finish'"
					@click:next="advanceStep"
					@click:prev="step--"
				/>
			</v-stepper>
		</v-card>
	</v-dialog>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { info } from 'tauri-plugin-log-api';
import { mdiDownload } from '@mdi/js';

import useSettings from '../composables/settings';
import useModStore from '../stores/mods';
import ModInstaller from './mods/ModInstaller.vue';
import ResonitePathSetting from './settings/ResonitePathSetting.vue';
import FieldCopyButton from './FieldCopyButton.vue';

const settings = useSettings();
const modStore = useModStore();
const showDialog = ref(true);
const step = ref(0);
const resonitePathSelected = computed(() =>
	Boolean(settings.current.resonitePath),
);
const prereqsInstalled = ref(0);

onMounted(() => {
	info('Setup guide showing');
	if (!modStore.mods && !modStore.loading) modStore.load();
});

/**
 * Advances the stepper to its next step, or on the final step, closes the dialog and stores that the setup guide is done
 */
function advanceStep() {
	if (step.value < 3) {
		step.value++;
	} else {
		showDialog.value = false;
		onModelChange(showDialog.value);

		// Save the necessary settings after allowing time for the dialog hide animation
		setTimeout(async () => {
			await settings.set('setupGuideDone', true, false);
			await settings.set('allowClosingSetupGuide', true, false);
			await settings.persist();
		}, 500);

		// Kick off mod autodiscovery
		if (!settings.current.modsAutodiscovered && !modStore.discovering) {
			modStore
				.discover()
				.then(() => {
					settings.set('modsAutodiscovered', true);
				})
				.catch(() => {});
		}
	}
}

/**
 * Handles model update events for the dialog
 * @param {boolean} shown
 */
function onModelChange(shown) {
	if (!shown) {
		setTimeout(() => {
			info('Setup guide done');
			settings.current.setupGuideDone = true;
		}, 500);
	}
}
</script>
