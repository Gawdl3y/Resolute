<template>
	<AppHeader title="Settings" />

	<v-main>
		<v-form>
			<v-container>
				<v-row>
					<v-col>
						<ResonitePathSetting />

						<TextSetting
							setting="manifestUrl"
							:rules="[rules.url]"
							label="Custom manifest URL"
						/>

						<DropdownSetting setting="theme" :items="themes" label="Theme" />

						<CheckboxSetting
							setting="groupMods"
							label="Group mods by category"
						/>

						<CheckboxSetting
							setting="modAuthorTools"
							label="Show mod authoring tools"
						/>

						<v-btn @click="settings.current.setupGuideDone = false">
							Show setup guide
						</v-btn>
					</v-col>
				</v-row>
			</v-container>
		</v-form>
	</v-main>
</template>

<script setup>
import useSettings from '../../composables/settings';
import AppHeader from '../AppHeader.vue';
import ResonitePathSetting from '../settings/ResonitePathSetting.vue';
import TextSetting from '../settings/TextSetting.vue';
import CheckboxSetting from '../settings/CheckboxSetting.vue';
import DropdownSetting from '../settings/DropdownSetting.vue';

const settings = useSettings();

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
