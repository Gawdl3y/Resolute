import { createApp } from 'vue';
import { createRouter, createWebHashHistory } from 'vue-router';
import { createPinia } from 'pinia';
import { attachConsole } from 'tauri-plugin-log-api';

import { disableContextMenu, disableTextSelection } from './util';
import App from './App.vue';
import Main from './components/pages/Main.vue';
import ModIndex from './components/pages/ModIndex.vue';
import Settings from './components/pages/Settings.vue';

import 'element-plus/theme-chalk/dark/css-vars.css';
import './styles/global.css';

attachConsole().then(() => {
	const router = createRouter({
		history: createWebHashHistory(),
		routes: [
			{ path: '/', component: Main },
			{ path: '/mods', component: ModIndex },
			{ path: '/settings', component: Settings },
		],
	});

	createApp(App)
		.use(router)
		.use(createPinia())
		.mount('#app');

	// Disable the context menu and text selection if we appear to be in a production build
	if(window.location.hostname === 'tauri.localhost') {
		disableContextMenu();
		disableTextSelection();
	}
});
