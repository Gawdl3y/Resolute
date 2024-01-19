<template>
	<SimpleTooltip v-slot="{ props: tooltipProps }" :text="tooltip">
		<v-btn v-bind="{ ...tooltipProps, ...$attrs }" :icon />
	</SimpleTooltip>
</template>

<script setup>
import SimpleTooltip from './SimpleTooltip.vue';

defineProps({
	icon: { type: String, required: true },
	tooltip: { type: String, required: true },
});
</script>
