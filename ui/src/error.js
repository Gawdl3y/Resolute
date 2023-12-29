import { createApp } from 'vue';
import { createVuetify } from 'vuetify';
import { aliases, mdi } from 'vuetify/iconsets/mdi-svg';

import ErrorApp from './ErrorApp.vue';

import 'vuetify/styles';
import '@fontsource/roboto-mono';
import './styles/global.css';

createApp(ErrorApp)
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
