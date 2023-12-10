import js from '@eslint/js';
import vue from 'eslint-plugin-vue';
import { FlatCompat } from '@eslint/eslintrc';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const compat = new FlatCompat({
	baseDirectory: __dirname,
});

export default [
	js.configs.recommended,
	...compat.extends('plugin:vue/vue3-recommended'),
	...compat.extends('plugin:prettier-vue/recommended'),

	{
		files: ['ui/src/**/*.{js,vue}'],
		plugins: {
			vue,
		},
		rules: {},
	},
];
