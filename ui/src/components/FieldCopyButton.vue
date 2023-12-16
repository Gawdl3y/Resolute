<template>
	<v-scroll-x-reverse-transition>
		<div v-if="copied" class="me-2">Copied!</div>
	</v-scroll-x-reverse-transition>

	<v-tooltip text="Copy" :open-delay="500" location="top">
		<template #activator="{ props: activator }">
			<v-slide-x-reverse-transition>
				<v-btn
					v-if="text && !hidden"
					v-bind="activator"
					variant="plain"
					:icon="mdiContentCopy"
					@click="copyText"
				/>
			</v-slide-x-reverse-transition>
		</template>
	</v-tooltip>
</template>

<script setup>
import { ref } from 'vue';
import { mdiContentCopy } from '@mdi/js';

const props = defineProps({
	text: { type: String, required: true },
	hidden: { type: Boolean, default: false },
});
const copied = ref(false);
let copiedTimeout = null;

/**
 * Writes the text to the clipboard
 */
function copyText() {
	navigator.clipboard.writeText(props.text);
	copied.value = true;

	if (copiedTimeout) clearTimeout(copiedTimeout);

	copiedTimeout = setTimeout(() => {
		copied.value = false;
		copiedTimeout = null;
	}, 2000);
}
</script>
