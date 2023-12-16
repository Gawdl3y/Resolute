<template>
	<AppHeader title="Settings" />

	<v-main>
		<v-form>
			<v-container>
				<v-row>
					<v-col>
						<!-- Resonite path setting -->
						<ResonitePathSelector />

						<!-- Custom manifest URL setting -->
						<v-text-field
							v-model="settings.current.manifestUrl"
							label="Custom manifest URL"
							variant="solo"
							:rules="[rules.url]"
							@blur="saveManifestUrl"
							@keypress.enter="saveManifestUrl"
						/>

						<!-- Theme setting -->
						<v-select
							v-model="settings.current.theme"
							label="Theme"
							variant="solo"
							:items="themes"
							item-title="name"
							item-value="val"
							@update:model-value="saveTheme"
						/>

						<!-- Group mods setting -->
						<CheckboxSetting
							setting="groupMods"
							label="Group mods by category"
						/>

						<!-- Show mod authoring tools setting -->
						<CheckboxSetting
							setting="modAuthorTools"
							label="Show mod authoring tools"
						/>

						<!-- Show setup guide button -->
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
import useSettings from '../../settings';
import AppHeader from '../AppHeader.vue';
import ResonitePathSelector from '../ResonitePathSelector.vue';
import CheckboxSetting from '../CheckboxSetting.vue';

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
 * Saves the custom manifest URL if valid
 */
async function saveManifestUrl() {
	const url = settings.current.manifestUrl;
	const valid = rules.url(url);
	if (!valid || typeof valid === 'string') return;

	if (url) await settings.store.set('manifestUrl', url);
	else await settings.store.delete('manifestUrl');
	await settings.store.save();
}

/**
 * Theme choices
 */
const themes = [
	{ name: 'System', val: null },
	{ name: 'Light', val: 'light' },
	{ name: 'Dark', val: 'dark' },
];

/**
 * Saves the theme selection
 */
async function saveTheme() {
	await settings.store.set('theme', settings.current.theme);
	await settings.store.save();
}
</script>
