import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const path = params.path;
	const result = await fetch(`/api/${path}/report.json`);
	const report = await result.json();
	return { report };
};

