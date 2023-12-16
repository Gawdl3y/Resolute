<template>
	<v-checkbox
		v-model="settings.current[setting]"
		:label="label"
		variant="solo"
		@update:model-value="save"
	/>
</template>

<script setup>
import useSettings from '../settings';

const props = defineProps({
	setting: { type: String, required: true },
	label: { type: String, required: true },
});
const settings = useSettings();

/**
 * Saves the checkbox setting
 */
async function save() {
	await settings.store.set(props.setting, settings.current[props.setting]);
	await settings.store.save();
}
</script>
