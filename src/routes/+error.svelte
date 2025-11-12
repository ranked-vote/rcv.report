<script lang="ts">
	import { page } from '$app/stores';

	export let status: number;
	export let error: Error;

	const dev = import.meta.env.DEV;
</script>

<svelte:head>
	<title>{status}</title>
</svelte:head>

<style>
	h1, p {
		margin: 0 auto;
	}

	h1 {
		font-size: 2.8em;
		font-weight: 700;
		margin: 0 0 0.5em 0;
	}

	p {
		margin: 1em auto;
	}

	@media (min-width: 480px) {
		h1 {
			font-size: 4em;
		}
	}
</style>

<h1>{status}</h1>

<p>{error?.message || 'An error occurred'}</p>

{#if dev && error?.stack}
	<pre>{error.stack}</pre>
{/if}

