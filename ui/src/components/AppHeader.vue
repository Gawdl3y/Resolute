<template>
	<v-app-bar :title="title" density="comfortable">
		<template #prepend>
			<v-btn
				:icon="mdiArrowLeft"
				:disabled="!canGoBack"
				@click="router.go(-1)"
			/>
		</template>

		<template v-if="$slots.actions" #append>
			<slot name="actions" />
		</template>

		<template v-if="$slots.extension" #extension>
			<slot name="extension" />
		</template>

		<v-progress-linear
			v-model="progress"
			:active="loading"
			:indeterminate="indeterminate"
			:stream="stream"
			absolute
			location="bottom"
		/>
	</v-app-bar>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { mdiArrowLeft } from '@mdi/js';

const progress = defineModel('progress', { type: Number, default: -1 });
defineProps({
	title: { type: String, required: true },
	loading: { type: Boolean, default: false },
	indeterminate: { type: Boolean, default: false },
	stream: { type: Boolean, default: false },
});

const router = useRouter();
const canGoBack = ref(Boolean(window.history.state.back));
let unregisterRouterHook = null;

onMounted(() => {
	unregisterRouterHook = router.afterEach(() => {
		canGoBack.value = Boolean(window.history.state.back);
	});
});

onUnmounted(() => {
	unregisterRouterHook();
});
</script>
