<script lang="ts">
	import { marked, type Token } from 'marked';
	import purify from 'dompurify';

	export let token: Token;

	const html = marked.parser([token]);
</script>

<div class="entry fade-in">{@html purify.sanitize(html)}</div>

<style>
	:global(.entry h1, h2, h3, h4, h5, h6) {
		font-weight: bold;
		margin-top: 0.4em;
		margin-bottom: 0.2em;
	}
	:global(.entry h1) {
		font-size: 1.4em;
	}
	:global(.entry h2) {
		font-size: 1.3em;
	}
	:global(.entry h3) {
		font-size: 1.2em;
	}

	:global(.entry ul),
	:global(.entry ol) {
		list-style-type: disc;
	}
	:global(.entry li) {
		margin-left: 0.5rem;
	}
	:global(.entry p) {
		margin-top: 1rem;
		margin-bottom: 1rem;
	}

	.fade-in {
		animation: fadeIn 1s;
	}
	@keyframes fadeIn {
		0% {
			opacity: 0;
		}
		100% {
			opacity: 1;
		}
	}
</style>
