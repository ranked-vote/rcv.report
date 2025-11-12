import { json } from '@sveltejs/kit';
import { getIndex } from '../../../reports.js';
import type { RequestHandler } from './$types';

export const prerender = true;

export const GET: RequestHandler = async () => {
	const index = await getIndex();
	return json(index);
};
