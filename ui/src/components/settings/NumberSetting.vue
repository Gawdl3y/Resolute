<template>
	<v-text-field
		v-model="settings.current[setting]"
		:label="label"
		:rules="effectiveRules"
		:variant="variant"
		:suffix="suffix"
		type="number"
		@keypress.enter="save"
		@blur="save"
	/>
</template>

<script setup>
import { computed } from 'vue';

import useSettings from '../../composables/settings';

const props = defineProps({
	setting: { type: String, required: true },
	label: { type: String, required: true },
	rules: { type: Array, default: null },
	variant: { type: String, default: 'solo' },
	max: { type: Number, default: Infinity },
	min: { type: Number, default: -Infinity },
	suffix: { type: String, default: undefined },
});

const settings = useSettings();

const numberRules = {
	float(val) {
		if (!val) return true;

		const number = Number(val);
		if (isNaN(number)) return 'Invalid number';

		return true;
	},

	min(minVal) {
		return function min(val) {
			if (val === null || val === '') return true;

			const number = Number(val);
			if (number < minVal) {
				return `Minimum of ${minVal}${props.suffix ? ` ${props.suffix}` : ''}`;
			}

			return true;
		};
	},

	max(maxVal) {
		return function max(val) {
			if (val === null || val === '') return true;

			const number = Number(val);
			if (number > maxVal) {
				return `Maximum of ${maxVal}${props.suffix ? ` ${props.suffix}` : ''}`;
			}

			return true;
		};
	},
};

const effectiveRules = computed(() => [
	...(props.rules ?? []),
	numberRules.float,
	numberRules.min(props.min),
	numberRules.max(props.max),
]);

/**
 * Validates then saves the text setting
 * @return {boolean} Whether the setting was valid and saved
 */
async function save() {
	const value = settings.current[props.setting];

	// Validate the input
	for (const rule of effectiveRules.value) {
		const valid = rule(value);
		if (!valid || typeof valid === 'string') return false;
	}

	if ((typeof value === 'string' && value) || typeof value === 'number') {
		await settings.set(props.setting, Number(value));
	} else {
		await settings.unset(props.setting);
	}

	return true;
}
</script>
