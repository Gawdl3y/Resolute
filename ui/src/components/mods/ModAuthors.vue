<template>
	<div class="d-flex flex-wrap ga-4 mb-8">
		<v-card
			v-for="author of authors"
			:key="author.name"
			:title="author.name"
			:href="author.url"
			target="_blank"
			variant="tonal"
			density="comfortable"
		>
			<template #prepend>
				<v-avatar v-if="author.icon" :image="author.icon" size="large" />
				<v-avatar
					v-else
					:icon="mdiAccount"
					size="large"
					color="surface-variant"
				/>
			</template>

			<template #append>
				<IconButton
					v-if="author.support"
					:tooltip="`Support ${author.name}`"
					:icon="mdiGift"
					:href="author.support"
					target="_blank"
					variant="text"
				/>
			</template>
		</v-card>
	</div>
</template>

<script setup>
import { mdiAccount, mdiGift } from '@mdi/js';

import IconButton from '../IconButton.vue';

defineProps({ authors: { type: Array, required: true } });
</script>
