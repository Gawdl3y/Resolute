<template>
	<v-tooltip
		v-model="showTooltip"
		:text="copied ? 'Copied!' : 'Copy'"
		:open-delay="500"
	>
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
import { ref, watch } from 'vue';
import { mdiContentCopy } from '@mdi/js';

const props = defineProps({
	text: { type: String, required: true },
	hidden: { type: Boolean, default: false },
});
const copied = ref(false);
const showTooltip = ref(false);

/**
 * Writes the text to the clipboard
 */
function copyText() {
	navigator.clipboard.writeText(props.text);
	copied.value = true;
	showTooltip.value = true;
}

watch(showTooltip, (_, show) => {
	if (show) {
		setTimeout(() => {
			copied.value = false;
		}, 350);
	}
});
</script>
