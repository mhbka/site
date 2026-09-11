// @ts-check

import mdx from '@astrojs/mdx';
import react from '@astrojs/react';
import sitemap from '@astrojs/sitemap';
import { defineConfig, fontProviders } from 'astro/config';
import { readFileSync } from 'node:fs';
import { parseEnv } from 'node:util';

import node from '@astrojs/node';

const exposedEnvPrefixes = ['PUBLIC_', 'SUPABASE_', 'BACKEND_'];
const isBuild = process.argv.includes('build');

function loadBuildEnv() {
	const env = parseEnv(readFileSync(new URL('./.env', import.meta.url), 'utf8'));

	return Object.fromEntries(
		Object.entries(env)
			.filter(([name]) => exposedEnvPrefixes.some((prefix) => name.startsWith(prefix)))
			.map(([name, value]) => [`import.meta.env.${name}`, JSON.stringify(value)]),
	);
}

// https://astro.build/config
export default defineConfig({
    site: 'https://kyunkyun.moe',
	integrations: [mdx(), react(), sitemap()],
	output: 'server',
	// The Supabase URL and publishable key are deliberately available to browser scripts.
	vite: {
		envPrefix: exposedEnvPrefixes,
		// Vite normally loads .env.local for every mode. Production builds must use
		// only .env, while local development keeps Vite's standard .env.local override.
		...(isBuild
			? {
				envDir: false,
				define: loadBuildEnv(),
			}
			: {}),
	},

	fonts: [
		{
			provider: fontProviders.local(),
			name: 'IBM Plex Sans',
			cssVariable: '--font-ibm-plex-sans',
			fallbacks: ['sans-serif'],
			options: {
				variants: [
					{ src: ['./src/assets/fonts/ibm-plex-sans-regular.woff2'], weight: 400, style: 'normal', display: 'swap' },
					{ src: ['./src/assets/fonts/ibm-plex-sans-bold.woff2'], weight: 700, style: 'normal', display: 'swap' },
				],
			},
		},
	],

  adapter: node({
    mode: 'standalone',
  }),
});
