<template>
	<AppHeader title="Dashboard" />

	<v-main class="d-flex flex-column align-center justify-center">
		<h1 class="text-h2 font-weight-black text-primary mb-6">WIP</h1>

		<v-sheet rounded elevation="4" class="w-75 pa-4" style="max-width: 32em">
			<div class="d-flex justify-space-between mb-3">
				<h2 class="text-h5">What the HECK is Resolute?</h2>
				<p v-if="version" class="text-disabled">v{{ version }}</p>
			</div>

			<p class="mb-3">
				Resolute is a mod manager for
				<a href="https://resonite.com/" target="_blank">Resonite</a>. It's still
				very much a work in progress, but the goal is to build a friendly,
				easy-to-use GUI for finding, installing, and managing your Resonite
				mods.
			</p>
			<p class="mb-6">
				At the moment, Resolute is capable of installing mods and not much else
				- although that's already much nicer than doing it by hand! In the
				future, it will be able to keep track of the installed mods, auto-update
				them, auto-install required mods, and more!
			</p>

			<div class="d-flex justify-space-between text-disabled">
				<p>
					Brought to you by
					<a href="https://gawdl3y.dev" target="_blank">Gawdl3y</a>, a huge
					nerd.
				</p>
				<p>
					<a href="https://github.com/Gawdl3y/Resolute" title="Source code">
						<v-icon :icon="mdiSourceBranch" />
					</a>
				</p>
			</div>
		</v-sheet>
	</v-main>
</template>

<script setup>
import { ref, onBeforeMount } from 'vue';
import { getVersion } from '@tauri-apps/api/app';
import { mdiSourceBranch } from '@mdi/js';
import AppHeader from '../AppHeader.vue';

const version = ref(null);

onBeforeMount(async () => {
	version.value = await getVersion();
});
</script>
