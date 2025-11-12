import { json, error } from '@sveltejs/kit';
import { getReport } from '../../../../reports.js';
import type { RequestHandler } from './$types';

export const prerender = true;

export const GET: RequestHandler = async ({ params }) => {
	const { path } = params;

	try {
		const report = await getReport(path);

		if (!report || !report.info) {
			throw error(404, 'Report not found');
		}

		return json(report);
	} catch (err: unknown) {
		if (err && typeof err === 'object' && 'status' in err) {
			throw err;
		}
		const message = err instanceof Error ? err.message : 'Internal server error';
		throw error(500, message);
	}
};
