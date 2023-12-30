<template>
	<v-tooltip :text="label" :open-delay="500" :disabled="expanded">
		<template #activator="{ props }">
			<v-list-item
				:title="label"
				:prepend-icon="icon"
				:to="path"
				v-bind="props"
			/>
		</template>
	</v-tooltip>
</template>

<script setup>
defineProps({
	label: { type: String, required: true },
	path: { type: String, required: true },
	icon: { type: String, required: true },
	expanded: { type: Boolean, required: false },
});
</script>
