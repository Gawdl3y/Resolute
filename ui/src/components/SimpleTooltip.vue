<template>
	<v-tooltip :text :open-delay="openDelay">
		<template #activator="{ props }">
			<slot :props />
		</template>
	</v-tooltip>
</template>

<script setup>
defineProps({
	text: { type: String, required: true },
	openDelay: { type: Number, default: 500 },
});
</script>
