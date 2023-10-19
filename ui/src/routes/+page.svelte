<script lang="ts">
	import {
		Button,
		Textarea,
		Footer,
		FooterCopyright,
		FooterLinkGroup,
		FooterLink
	} from 'flowbite-svelte';
	import { EventSourceParserStream } from 'eventsource-parser/stream';
	import type { ParsedEvent } from 'eventsource-parser';
	import NavBar from '../components/NavBar.svelte';
	import { marked, type Token } from 'marked';
	import Markdown from '../components/Markdown.svelte';
	import { writable, type Writable } from 'svelte/store';
	import Pulser from '../components/Pulser.svelte';

	let jobRequirements = ``;
	let extendedResume = ``;

	let tokens = 0;
	let buffer = '';
	let markdownTokens: Writable<Token[]> = writable([]);
	let running = false;

	async function refineResume(e: MouseEvent): Promise<void> {
		if (jobRequirements === '' || extendedResume === '') {
			console.log('please fill in both fields');
			processBuffer('Please fill in both fields\n');
			return;
		}
		running = true;

		const response = await fetch('/api/refine', {
			redirect: 'manual',
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				Accept: 'text/event-stream' // Indicate that we expect an SSE stream
			},
			body: JSON.stringify({
				jobRequirement: jobRequirements,
				extendedResume: extendedResume
			})
		});

		// Check if the request was successful
		if (!response.ok) {
			console.error('Failed to refine resume:', response.statusText);
			processBuffer('Error. Please make sure that you are logged in.\n');
			return;
		}

		const body = response.body;
		if (body) {
			markdownTokens.set([]);
			const eventStream = response.body
				.pipeThrough(new TextDecoderStream())
				.pipeThrough(new EventSourceParserStream())
				.getReader();

			for (;;) {
				const { done, value } = await eventStream.read();
				if (done) break;

				const event = value as ParsedEvent;
				if (event.event === 'message') {
					if (event.data === '[DONE]') {
						buffer += '\n';
						buffer = processBuffer(buffer);
						running = false;
						return;
					}
					buffer += event.data;
					buffer = processBuffer(buffer);
				}
				if (event.event === 'tokens') {
					console.log('Tokens:', event.data);
					tokens = parseInt(event.data, 10);
				}
			}
		}
	}

	function updateTokens(newTokens: Token[]) {
		markdownTokens.update((oldTokens) => {
			oldTokens.push(...newTokens);
			console.log({ tokens: oldTokens });
			return oldTokens;
		});
	}

	function processBuffer(buffer: string): string {
		let firstNewline = buffer.indexOf('\n');
		while (firstNewline !== -1) {
			const line = buffer.slice(0, firstNewline);
			buffer = buffer.slice(firstNewline + 1);
			const tokens = marked.lexer(line);
			updateTokens(tokens);
			firstNewline = buffer.indexOf('\n');
		}
		return buffer;
	}
</script>

<NavBar {tokens} />

<div class="mt-10 w-full flex flex-col justify-center items-center">
	<div class="w-4/5">
		<div>
			<Textarea
				class="resize-none"
				rows="1"
				placeholder="Paste Job Description and requirements Here..."
				bind:value={jobRequirements}
				style="height: 12em; max-height: 20em; overflow-y: auto;"
			/>
		</div>
	</div>
	<div class="w-4/5">
		<div>
			<Textarea
				class="resize-none"
				rows="1"
				placeholder="Paste Your current resume here. Make this as descriptive as possible..."
				bind:value={extendedResume}
				style="height: 12em; max-height: 20em; overflow-y: auto;"
			/>
		</div>
	</div>
	<div class="flex justify-center mt-2">
		<Button on:click={refineResume}>Refine!</Button>
	</div>
	<div class="w-4/5 mt-2">
		<!-- Display the refined content rendered from Markdown -->
		<div class="card">
			{#each $markdownTokens as token}
				<Markdown {token} />
			{/each}
			{#if running}
				<Pulser />
			{/if}
		</div>
	</div>
</div>
<Footer class="absolute bottom-0 left-0 z-20 w-full">
	<FooterCopyright href="/" by="Candle" year={2023} />
	<FooterLinkGroup
		ulClass="flex flex-wrap items-center mt-3 text-sm text-gray-500 dark:text-gray-400 sm:mt-0"
	>
		<FooterLink href="https://github.com/candlecorp/resumerefiner.com">Source</FooterLink>
		<FooterLink href="https://candle.dev/privacy.html">Privacy Policy</FooterLink>
		<FooterLink href="https://discord.gg/candle">Discord</FooterLink>
	</FooterLinkGroup>
</Footer>

<style>
	/* Add styling for the Card, if you'd like */
	.card {
		padding: 16px;
		border: 1px solid #ddd;
		border-radius: 4px;
	}
</style>
