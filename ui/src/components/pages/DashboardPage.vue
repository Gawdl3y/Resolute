<template>
	<AppHeader title="Dashboard" />

	<v-main class="d-flex flex-column align-center justify-center">
		<h1 class="text-h2 font-weight-black text-primary mb-6">WIP</h1>

		<v-sheet rounded elevation="4" class="w-75 pa-4" style="max-width: 32em">
			<div class="d-flex justify-space-between mb-3">
				<h2 class="text-h5">What the HECK is Resolute?</h2>
				<p class="text-body-1 text-disabled">v{{ app.version.value }}</p>
			</div>

			<p class="mb-3 text-body-1">
				Resolute is a mod manager for
				<a href="https://resonite.com/" target="_blank">Resonite</a>. It's still
				very much a work in progress, but the goal is to build a friendly,
				easy-to-use GUI for finding, installing, and managing your Resonite
				mods.
			</p>
			<p class="mb-6 text-body-1">
				Perhaps this dashboard page will have a bigger purpose in the
				future&hellip; Only time will tell!
			</p>

			<div class="d-flex justify-space-between text-body-1 text-disabled">
				<p>
					Brought to you by
					<a href="https://gawdl3y.dev" target="_blank">Gawdl3y</a>, a huge
					nerd.
				</p>
				<p>
					<a
						href="https://github.com/Gawdl3y/Resolute"
						target="_blank"
						title="Source code"
					>
						<v-icon :icon="mdiSourceBranch" />
					</a>
				</p>
			</div>
		</v-sheet>
	</v-main>
</template>

<script setup>
import { mdiSourceBranch } from '@mdi/js';

import useApp from '../../composables/app';
import AppHeader from '../AppHeader.vue';

const app = useApp();
</script>
