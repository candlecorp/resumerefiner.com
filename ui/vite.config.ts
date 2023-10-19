import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],

	server: {
		strictPort: true,
		port: 5173,
		hmr: {
			host: 'localhost',
			clientPort: 5173
		}
	}
});
