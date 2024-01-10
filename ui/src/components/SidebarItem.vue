<template>
	<SimpleTooltip v-slot="{ props }" :text="label" :disabled="expanded">
		<v-list-item
			v-bind="props"
			:title="label"
			:prepend-icon="icon"
			:to="path"
		/>
	</SimpleTooltip>
</template>

<script setup>
import SimpleTooltip from './SimpleTooltip.vue';

defineProps({
	label: { type: String, required: true },
	path: { type: String, required: true },
	icon: { type: String, required: true },
	expanded: { type: Boolean, required: false },
});
</script>
