import { createApp } from 'vue';
import { createRouter, createWebHashHistory } from 'vue-router';
import { createPinia } from 'pinia';
import { createVuetify } from 'vuetify';
import { aliases, mdi } from 'vuetify/iconsets/mdi-svg';
import { attachConsole } from 'tauri-plugin-log-api';

import { disableContextMenu, disableTextSelection } from './util';
import App from './App.vue';
import DashboardPage from './components/pages/DashboardPage.vue';
import ModsPage from './components/pages/ModsPage.vue';
import SettingsPage from './components/pages/SettingsPage.vue';

import 'vuetify/styles';
import './styles/global.css';

attachConsole().then(() => {
	const router = createRouter({
		history: createWebHashHistory(),
		routes: [
			{ path: '/', component: DashboardPage },
			{ path: '/mods', component: ModsPage },
			{ path: '/settings', component: SettingsPage },
		],
	});

	createApp(App)
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
