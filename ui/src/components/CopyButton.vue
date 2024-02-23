<template>
	<TextCopier v-slot="{ props: copierProps, copy }" :text>
		<v-slide-x-reverse-transition>
			<v-btn
				v-if="text && !hidden"
				v-bind="{ ...copierProps, ...$attrs }"
				variant="plain"
				:icon="mdiContentCopy"
				@click="copy()"
			/>
		</v-slide-x-reverse-transition>
	</TextCopier>
</template>

<script setup>
import { mdiContentCopy } from '@mdi/js';

import TextCopier from './TextCopier.vue';

defineProps({
	text: { type: String, required: true },
	hidden: { type: Boolean, default: false },
});
</script>
