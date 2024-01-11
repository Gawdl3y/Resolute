<template>
	<slot v-if="!tooltip" :copy="copy" :copied="copied" :reset="reset" />

	<SimpleTooltip
		v-else
		v-slot="{ props: tooltipProps }"
		v-model="showTooltip"
		:text="copied ? 'Copied!' : 'Copy'"
	>
		<slot :copy="copy" :copied="copied" :reset="reset" :props="tooltipProps" />
	</SimpleTooltip>
</template>

<script setup>
import { ref, watch } from 'vue';

import SimpleTooltip from './SimpleTooltip.vue';

defineExpose({ copy, reset });
const props = defineProps({
	text: { type: String, required: true },
	tooltip: { type: Boolean, default: true },
});

/**
 * Whether the text has been copied
 */
const copied = ref(false);

/**
 * Whether the tooltip is being shown
 */
const showTooltip = ref(false);

// Reset the copied state whenever the tooltip closes
watch(showTooltip, (show) => {
	if (!show) setTimeout(reset, 350);
});

/**
 * Writes the text to the clipboard and sets the copied state
 * @param {boolean} [forceShowTooltip=true]
 */
function copy(forceShowTooltip = true) {
	navigator.clipboard.writeText(props.text);
	copied.value = true;
	if (forceShowTooltip) showTooltip.value = true;
}

/**
 * Resets the copied state
 */
function reset() {
	copied.value = false;
}
</script>
