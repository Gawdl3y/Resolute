<template>
	<ModsPage
		title="Mod Index"
		:mods="modStore.mods"
		:load-mods="modStore.load"
		:disabled="!resonitePathExists"
	>
		<template #alerts>
			<v-alert
				v-if="!resonitePathExists"
				type="warning"
				:rounded="false"
				density="comfortable"
				class="rounded-0"
			>
				{{
					resonitePathExists === null
						? 'Please configure the Resonite path in the settings.'
						: "The configured Resonite path doesn't seem to exist. Please check the settings."
				}}
			</v-alert>
		</template>
	</ModsPage>
</template>

<script setup>
import { ref, watch, onBeforeMount } from 'vue';
import { invoke } from '@tauri-apps/api';

import useSettings from '../../composables/settings';
import useModStore from '../../stores/mods';
import ModsPage from './ModsPage.vue';

const settings = useSettings();
const modStore = useModStore();
const resonitePathExists = ref(true);

onBeforeMount(checkIfResonitePathExists);
watch(settings.current, checkIfResonitePathExists);

/**
 * Checks whether the Resonite path is configured and exists via the backend
 */
async function checkIfResonitePathExists() {
	if (!settings.current.resonitePath) {
		resonitePathExists.value = null;
	} else {
		resonitePathExists.value = await invoke('verify_resonite_path').catch(
			() => false,
		);
	}
}
</script>
