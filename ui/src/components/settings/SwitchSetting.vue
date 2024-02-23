<template>
	<v-switch
		v-model="settings.current[setting]"
		:label
		color="primary"
		hide-details
		@update:model-value="save"
	/>
</template>

<script setup>
import useSettings from '../../composables/settings';

const props = defineProps({
	setting: { type: String, required: true },
	label: { type: String, required: true },
});
const settings = useSettings();

/**
 * Saves the checkbox setting
 */
async function save() {
	await settings.set(props.setting, settings.current[props.setting]);
}
</script>
