<template>
	<AppHeader title="All Mods">
		<template #actions>
			<v-tooltip text="Refresh mods" :open-delay="500">
				<template #activator="{ props }">
					<v-btn
						:icon="mdiRefresh"
						v-bind="props"
						@click="modStore.load(true)"
					/>
				</template>
			</v-tooltip>
		</template>
	</AppHeader>

	<v-main>
		<v-alert
			v-if="!resonitePathExists"
			type="warning"
			:rounded="false"
			class="rounded-0"
		>
			{{
				resonitePathExists === null
					? 'Please configure the Resonite path in the settings.'
					: "The configured Resonite path doesn't seem to exist. Mod installation will probably fail."
			}}
		</v-alert>

		<ModTable
			:mods="modStore.mods"
			:disabled="resonitePathExists === null"
			style="height: 100%"
		/>
	</v-main>
</template>

<script setup>
import { ref, onBeforeMount, onMounted } from 'vue';
import { exists as fsExists } from '@tauri-apps/api/fs';
import { mdiRefresh } from '@mdi/js';

import useSettings from '../../settings';
import useModStore from '../../stores/mods';
import AppHeader from '../AppHeader.vue';
import ModTable from '../ModTable.vue';

const settings = useSettings();
const modStore = useModStore();
const resonitePathExists = ref(true);

onBeforeMount(async () => {
	if (!settings.current.resonitePath) {
		resonitePathExists.value = null;
	} else {
		resonitePathExists.value = await fsExists(settings.current.resonitePath);
	}
});

onMounted(() => {
	if (!modStore.mods) modStore.load();
});
</script>
