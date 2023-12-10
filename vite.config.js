import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import AutoImport from 'unplugin-auto-import/vite';
import Components from 'unplugin-vue-components/vite';
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers';
import Icons from 'unplugin-icons/vite';
import IconsResolver from 'unplugin-icons/resolver';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
	root: './ui',
	plugins: [
		vue(),
		AutoImport({
			resolvers: [ElementPlusResolver(), IconsResolver({ prefix: 'Icon' })],
		}),
		Components({
			resolvers: [
				ElementPlusResolver(),
				IconsResolver({ enabledCollections: ['ep'] }),
			],
		}),
		Icons({
			autoInstall: true,
		}),
	],

	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
	},
}));
