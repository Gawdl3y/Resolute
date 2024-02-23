<template>
	<slot :update :busy :updating :updated :error />
</template>

<script setup>
import { ref, computed } from 'vue';

import useModStore from '../../stores/mods';

const props = defineProps({
	mod: {
		required: true,
		validator(val) {
			if (!val) return false;
			return typeof val === 'object' || typeof val === 'string';
		},
	},
	version: { type: String, default: undefined },
});
const emit = defineEmits(['update', 'error']);

const modStore = useModStore();
const busy = computed(() => modStore.isBusy(props.mod));
const updating = computed(() => modStore.isUpdating(props.mod));
const updated = ref(false);
const error = ref(null);

/**
 * Updates the mod and updates internal state as needed
 */
async function update() {
	try {
		await modStore.update(props.mod, props.version);
		updated.value = true;
		emit('update');
	} catch (err) {
		error.value = err;
		emit('error', err);
		throw error;
	}
}
</script>
