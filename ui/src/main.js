import { createApp } from 'vue';
import { attachConsole } from 'tauri-plugin-log-api';

import { disableContextMenu, disableTextSelection } from './util';
import App from './App.vue';
import 'element-plus/theme-chalk/dark/css-vars.css';
import './styles/global.css';

attachConsole().then(() => {
	createApp(App).mount('#app');

	// Disable the context menu and text selection if we appear to be in a production build
	if(window.location.hostname === 'tauri.localhost') {
		disableContextMenu();
		disableTextSelection();
	}
});
