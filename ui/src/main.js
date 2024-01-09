import { createApp } from 'vue';
import { createRouter, createWebHashHistory } from 'vue-router';
import { createPinia } from 'pinia';
import { createVuetify } from 'vuetify';
import { aliases, mdi } from 'vuetify/iconsets/mdi-svg';
import { attachConsole } from 'tauri-plugin-log-api';

import { disableContextMenu, disableTextSelection } from './util';
import AppWrapper from './AppWrapper.vue';
import DashboardPage from './components/pages/DashboardPage.vue';
import AllModsPage from './components/pages/AllModsPage.vue';
import InstalledModsPage from './components/pages/InstalledModsPage.vue';
import ModAuthorToolsPage from './components/pages/ModAuthorToolsPage.vue';
import SessionLogPage from './components/pages/SessionLogPage.vue';
import SettingsPage from './components/pages/SettingsPage.vue';

import '@fontsource/roboto-mono/300.css';
import '@fontsource/roboto-mono/400.css';
import 'vuetify/styles';
import './styles/global.css';

const debug = window.location.hostname === 'tauri.localhost';

(debug ? attachConsole() : Promise.resolve()).then(() => {
	const router = createRouter({
		history: createWebHashHistory(),
		routes: [
			{ path: '/', component: DashboardPage },
			{ path: '/mods', component: AllModsPage },
			{ path: '/mods/installed', component: InstalledModsPage },
			{ path: '/author-tools', component: ModAuthorToolsPage },
			{ path: '/log', component: SessionLogPage },
			{ path: '/settings', component: SettingsPage },
		],
	});

	createApp(AppWrapper)
		.use(router)
		.use(createPinia())
		.use(
			createVuetify({
				icons: {
					defaultSet: 'mdi',
					aliases,
					sets: { mdi },
				},
			}),
		)
		.mount('#app');

	// Disable the context menu and text selection if we appear to be in a production build
	if (window.location.hostname === 'tauri.localhost') {
		disableContextMenu();
		disableTextSelection();
	}
});
