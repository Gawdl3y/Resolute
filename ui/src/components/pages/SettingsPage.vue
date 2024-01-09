<template>
	<AppHeader title="Settings">
		<template #extension>
			<v-tabs v-model="tab">
				<v-tab value="general" tabindex="0">General</v-tab>
				<v-tab value="advanced">Advanced</v-tab>
			</v-tabs>
		</template>
	</AppHeader>

	<v-main>
		<v-window v-model="tab">
			<v-window-item value="general">
				<v-container>
					<DropdownSetting setting="theme" :items="themes" label="Theme" />
					<ResonitePathSetting />
					<SwitchSetting
						setting="groupModIndex"
						label="Group Mod Index by category"
						class="mb-4"
					/>
					<v-btn @click="settings.current.setupGuideDone = false">
						Setup guide
					</v-btn>
				</v-container>
			</v-window-item>

			<v-window-item value="advanced">
				<v-container>
					<TextSetting
						setting="manifestUrl"
						:rules="[rules.url]"
						label="Custom manifest URL"
						hint="This will be used to list mods instead of the main Resonite Modding Group manifest"
					/>
					<NumberSetting
						setting="connectTimeout"
						:rules="[rules.required]"
						:min="5"
						:max="120"
						label="Connection timeout"
						suffix="seconds"
						hint="How long to wait before an attempted HTTP connection is considered failed"
					/>
					<SwitchSetting
						setting="modAuthorTools"
						label="Show mod authoring tools"
					/>
					<SwitchSetting setting="console" label="Show console" />
				</v-container>
			</v-window-item>
		</v-window>
	</v-main>
</template>

<script setup>
import { ref } from 'vue';

import useSettings from '../../composables/settings';
import AppHeader from '../AppHeader.vue';
import ResonitePathSetting from '../settings/ResonitePathSetting.vue';
import TextSetting from '../settings/TextSetting.vue';
import NumberSetting from '../settings/NumberSetting.vue';
import SwitchSetting from '../settings/SwitchSetting.vue';
import DropdownSetting from '../settings/DropdownSetting.vue';

const settings = useSettings();
const tab = ref('general');

/**
 * Validation rules
 */
const rules = {
	required(val) {
		if (val === null || val === '') return 'Required';
		return true;
	},

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
