import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const result = await fetch('/api/reports.json');
	const index = await result.json();
	return index;
};

