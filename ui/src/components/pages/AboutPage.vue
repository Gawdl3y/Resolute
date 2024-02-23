<template>
	<AppHeader title="About" />

	<v-main class="d-flex flex-column align-center justify-center">
		<img :src="logo" width="150" height="150" class="mb-4" />

		<h1 class="text-h2 font-weight-black text-resograd mb-2">Resolute</h1>
		<p class="text-subtitle text-disabled mb-6">v{{ app.version.value }}</p>

		<p class="text-body-1 mb-1">
			A mod manager for
			<a href="https://resonite.com/" target="_blank">Resonite</a>.
		</p>
		<p class="text-body-1 mb-6">
			Brought to you by
			<a href="https://gawdl3y.dev" target="_blank">Gawdl3y</a>, a huge nerd.
		</p>

		<div class="d-flex ga-4">
			<v-btn
				:prepend-icon="mdiGift"
				size="large"
				class="mt-6"
				href="https://ko-fi.com/gawdl3y"
				target="_blank"
			>
				Support Resolute
			</v-btn>

			<v-btn
				:prepend-icon="mdiSourceBranch"
				size="large"
				class="mt-6"
				href="https://github.com/Gawdl3y/Resolute"
				target="_blank"
			>
				Source Code
			</v-btn>
		</div>
	</v-main>
</template>

<script setup>
import { mdiSourceBranch, mdiGift } from '@mdi/js';

import logo from '../../assets/logo.png';
import useApp from '../../composables/app';
import AppHeader from '../AppHeader.vue';

const app = useApp();
</script>
