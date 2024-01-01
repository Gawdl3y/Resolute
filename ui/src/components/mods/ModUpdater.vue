<template>
	<slot
		:update="update"
		:busy="busy"
		:updating="updating"
		:updated="updated"
		:error="error"
	/>
</template>

<script setup>
import { ref, computed } from 'vue';

import useModStore from '../../stores/mods';

const props = defineProps({ mod: { type: Object, required: true } });
const emit = defineEmits(['update', 'error']);

const modStore = useModStore();
const busy = computed(() => modStore.isBusy(props.mod.id));
const updating = computed(() => modStore.isUpdating(props.mod.id));
const updated = ref(false);
const error = ref(null);

/**
 * Updates the mod and updates internal state as needed
 */
async function update() {
	try {
		await modStore.update(props.mod.id);
		updated.value = true;
		emit('update');
	} catch (err) {
		error.value = err;
		emit('error', err);
		throw error;
	}
}
</script>
