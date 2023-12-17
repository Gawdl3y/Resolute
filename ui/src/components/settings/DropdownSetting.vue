<template>
	<v-select
		v-model="settings.current[setting]"
		:label="label"
		:items="items"
		:variant="variant"
		item-title="label"
		item-value="value"
		@update:model-value="save"
	/>
</template>

<script setup>
import useSettings from '../../composables/settings';

const props = defineProps({
	setting: { type: String, required: true },
	items: { type: Array, required: true },
	label: { type: String, required: true },
	variant: { type: String, default: 'solo' },
});
const settings = useSettings();

/**
 * Saves the dropdown selection
 */
async function save() {
	await settings.set(props.setting, settings.current[props.setting]);
}
</script>
