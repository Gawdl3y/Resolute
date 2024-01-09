<template>
	<span
		class="clickable"
		tabindex="0"
		@click="action()"
		@keypress.space="action()"
		@keypress.enter="action()"
	>
		<slot />
	</span>
</template>

<script setup>
defineProps({ action: { type: Function, required: true } });
</script>

<style>
span.clickable {
	cursor: pointer;
}
</style>
