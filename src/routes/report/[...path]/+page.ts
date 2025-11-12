import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const path = params.path;
	const result = await fetch(`/api/${path}/report.json`);

	if (!result.ok) {
		const errorData = await result.json().catch(() => ({ error: 'Report not found' }));
		throw error(result.status, errorData.error || 'Report not found');
	}

	const report = await result.json();

	if (!report || !report.info) {
		throw error(404, 'Report not found');
	}

	return { report, path: path.split('/') };
};

