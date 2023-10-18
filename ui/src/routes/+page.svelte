<script lang="ts">
	import { Button, Textarea } from 'flowbite-svelte';
	import SvelteMarkdown from 'svelte-markdown';
	import { EventSourceParserStream } from 'eventsource-parser/stream';
	import type { ParsedEvent } from 'eventsource-parser';
	import NavBar from '../components/NavBar.svelte';

	let jobRequirements = '';
	let extendedResume = '';
	let renderedContent = '';
	let tokens = 0;

	async function refineResume(e: MouseEvent): Promise<void> {
		if (jobRequirements === '' || extendedResume === '') {
			console.log('please fill in both fields');
			return;
		}

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
			return;
		}

		const body = response.body;
		// debugger;
		if (body) {
			renderedContent = '';
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
						renderedContent += '';
						return;
					}
					renderedContent += event.data;
				}
				if (event.event === 'tokens') {
					console.log('Tokens:', event.data);
					tokens = event.data;
				}
			}
		}
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
			<SvelteMarkdown source={renderedContent} />
		</div>
	</div>
</div>

<style>
	/* Add styling for the Card, if you'd like */
	.card {
		padding: 16px;
		border: 1px solid #ddd;
		border-radius: 4px;
	}
</style>
