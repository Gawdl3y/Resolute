<template>
	<AppHeader title="Settings">
		<template #extension>
			<v-tabs v-model="tab">
				<v-tab value="general">General</v-tab>
				<v-tab value="advanced">Advanced</v-tab>
			</v-tabs>
		</template>
	</AppHeader>

	<v-main>
		<v-container>
			<v-window v-model="tab">
				<v-window-item value="general">
					<DropdownSetting setting="theme" :items="themes" label="Theme" />
					<ResonitePathSetting />
					<CheckboxSetting setting="groupMods" label="Group mods by category" />
					<v-btn @click="settings.current.setupGuideDone = false">
						Setup guide
					</v-btn>
				</v-window-item>

				<v-window-item value="advanced">
					<TextSetting
						setting="manifestUrl"
						:rules="[rules.url]"
						label="Custom manifest URL"
						hint="This will be used to list mods instead of the main Resonite Modding Group manifest"
					/>
					<CheckboxSetting
						setting="modAuthorTools"
						label="Show mod authoring tools"
					/>
				</v-window-item>
			</v-window>
		</v-container>
	</v-main>
</template>

<script setup>
import { ref } from 'vue';

import useSettings from '../../composables/settings';
import AppHeader from '../AppHeader.vue';
import ResonitePathSetting from '../settings/ResonitePathSetting.vue';
import TextSetting from '../settings/TextSetting.vue';
import CheckboxSetting from '../settings/CheckboxSetting.vue';
import DropdownSetting from '../settings/DropdownSetting.vue';

const settings = useSettings();
const tab = ref('general');

/**
 * Validation rules
 */
const rules = {
	url(val) {
		if (!val) return true;

		try {
			return Boolean(new URL(val));
		} catch (_err) {
			return 'Invalid URL';
		}
	},
};

/**
 * Theme choices
 */
const themes = [
	{ label: 'System', value: null },
	{ label: 'Light', value: 'light' },
	{ label: 'Dark', value: 'dark' },
];
</script>
