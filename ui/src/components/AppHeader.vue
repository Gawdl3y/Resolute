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
	</v-app-bar>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { mdiArrowLeft } from '@mdi/js';

defineProps({ title: { type: String, required: true } });

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
