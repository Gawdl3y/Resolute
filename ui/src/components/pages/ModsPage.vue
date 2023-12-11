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
		<ModTable :mods="modStore.mods" style="height: 100%" />
	</v-main>
</template>

<script setup>
import { onMounted } from 'vue';
import { mdiRefresh } from '@mdi/js';

import useModStore from '../../stores/mods';
import AppHeader from '../AppHeader.vue';
import ModTable from '../ModTable.vue';

const modStore = useModStore();

onMounted(() => {
	if (!modStore.mods) modStore.load();
});
</script>
