import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
	resolve: {
		alias: {
			$styles: path.resolve(__dirname, './src/styles')
		}
	},
	plugins: [sveltekit()]
});
