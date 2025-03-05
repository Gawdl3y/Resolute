import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import vuetify from 'vite-plugin-vuetify';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
	root: './ui',
	plugins: [vue(), vuetify({ autoImport: true })],

	clearScreen: false,
	server: {
		host: host || false,
		port: 1420,
		strictPort: true,
		hmr: host
			? {
					protocol: 'ws',
					host: host,
					port: 1430,
				}
			: undefined,
	},
}));
