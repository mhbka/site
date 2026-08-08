// @ts-check

import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';
import { defineConfig, fontProviders } from 'astro/config';
import { readFileSync } from 'node:fs';
import { parseEnv } from 'node:util';

import node from '@astrojs/node';

const exposedEnvPrefixes = ['PUBLIC_', 'SUPABASE_'];
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
    site: 'https://example.com',
    integrations: [mdx(), sitemap()],
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
          name: 'Atkinson',
          cssVariable: '--font-atkinson',
          fallbacks: ['sans-serif'],
          options: {
              variants: [
                  {
                      src: ['./src/assets/fonts/atkinson-regular.woff'],
                      weight: 400,
                      style: 'normal',
                      display: 'swap',
                  },
                  {
                      src: ['./src/assets/fonts/atkinson-bold.woff'],
                      weight: 700,
                      style: 'normal',
                      display: 'swap',
                  },
              ],
          },
      },
	],

  adapter: node({
    mode: 'standalone',
  }),
});
