import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { readFileSync } from 'fs';
import { resolve } from 'path';

const RANKED_VOTE_REPORTS = process.env.RANKED_VOTE_REPORTS
	? resolve(process.env.RANKED_VOTE_REPORTS)
	: resolve('./report_pipeline/reports');

function getPrerenderEntries() {
	try {
		const indexRaw = readFileSync(`${RANKED_VOTE_REPORTS}/index.json`, 'utf8');
		const index = JSON.parse(indexRaw);
		const entries = [];

		// Generate entries for all report and card routes
		for (const election of index.elections || []) {
			for (const contest of election.contests || []) {
				const path = `${election.path}/${contest.office}`;
				entries.push(`/report/${path}`);
				entries.push(`/card/${path}`);
			}
		}

		return entries;
	} catch (err) {
		console.warn('Could not load reports index for prerender entries:', err);
		return [];
	}
}

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess({ script: true }),

	kit: {
		adapter: adapter({
			pages: 'build',
			assets: 'build',
			fallback: undefined,
			precompress: false,
			strict: true
		}),
		prerender: {
			entries: ['*', ...getPrerenderEntries()],
			handleHttpError: ({ path, referrer, message }) => {
				// Ignore errors for card routes during prerender - they're generated dynamically
				if (path.startsWith('/card/')) {
					console.warn(`Skipping prerender for ${path}: ${message}`);
					return;
				}
				// For other routes, throw to fail the build
				throw new Error(`Prerender error for ${path}: ${message}`);
			}
		}
	}
};

export default config;

