<template>
	<v-text-field
		v-model="settings.current[setting]"
		:label="label"
		:rules="rules"
		:variant="variant"
		@keypress.enter="save"
		@blur="save"
	/>
</template>

<script setup>
import useSettings from '../../composables/settings';

const props = defineProps({
	setting: { type: String, required: true },
	label: { type: String, required: true },
	rules: { type: Array, default: null },
	variant: { type: String, default: 'solo' },
});
const settings = useSettings();

/**
 * Validates then saves the text setting
 * @return {boolean} Whether the setting was valid and saved
 */
async function save() {
	const value = settings.current[props.setting];

	// Validate the input
	if (props.rules) {
		for (const rule of props.rules) {
			const valid = rule(value);
			if (!valid || typeof valid === 'string') return false;
		}
	}

	if (value) await settings.set(props.setting, value);
	else await settings.unset(props.setting);

	return true;
}
</script>
