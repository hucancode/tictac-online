import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
	resolve: {
		alias: {
			$icons: path.resolve(__dirname, './src/icons'),
			$styles: path.resolve(__dirname, './src/styles'),
			$components: path.resolve(__dirname, './src/components'),
			$lib: path.resolve(__dirname, './src/lib'),
		}
	},
	plugins: [sveltekit()]
});
