<script lang="ts">
	import { Navbar, NavBrand, Button, Popover, Progressbar, Alert } from 'flowbite-svelte';
	import { QuestionCircleSolid } from 'flowbite-svelte-icons';
	import { onMount } from 'svelte';
	import { API } from '../lib/api';
	import type { UserInfo, TokenUsage, ConfirmStar, Stargazer } from '../lib/types';

	let loggedIn = false;
	export let tokens = 0;
	let stargazer = false;
	let api = new API();
	let user_info: UserInfo;
	let daily_usage = 0;
	let token_usage: TokenUsage = {
		daily_usage_value: 0,
		entitlement_value: 100,
		entitlement_name: 'tokens'
	};
	let stargazer_response = false;
	let stargazer_response_body = '';

	onMount(async () => {
		user_info = await api.getUserInfo();
		if (user_info) {
			console.log(user_info);
			if (user_info.email != null) {
				loggedIn = true;
			}
		}
		is_stargazer();
		dailyUsage();
	});

	async function is_stargazer() {
		let confirm_star: Stargazer = await api.isStargazer();
		if (confirm_star) {
			console.log(confirm_star);
			stargazer = confirm_star.is_stargazer;
		}
	}

	async function dailyUsage() {
		let daily_usage = await api.getDailyUsage();
		if (daily_usage) {
			console.log(daily_usage);
			token_usage = daily_usage;
		}
	}

	async function checkStargazer() {
		stargazer_response = false;
		let confirm_star: ConfirmStar = await api.confirmStar();
		if (confirm_star) {
			console.log(confirm_star);
			stargazer_response_body = confirm_star.message;
			stargazer_response = true;
			if (confirm_star.status) stargazer = true;
			dailyUsage();
		}
	}

	$: {
		daily_usage =
			((+token_usage.daily_usage_value + +tokens) / +token_usage.entitlement_value) * 100;
		if (daily_usage > 100) {
			daily_usage = 100;
		}
	}
</script>

<Navbar>
	<NavBrand href="/">
		<img src="wick_logo.png" class="mr-3 h-6 sm:h-9" alt="Wick Logo" />
	</NavBrand>
	<div class="text-center">
		<div>Tokens<QuestionCircleSolid id="tok" class="float-right ml-2" size="lg" /></div>
		<Popover
			triggeredBy="#tok"
			class="w-64 text-sm font-light text-gray-500 bg-white dark:text-gray-400 dark:border-gray-600 dark:bg-gray-800"
		>
			<div class="p-3">
				{#if !stargazer}
					<div class="mb-4 text-sm font-light">
						Want more tokens? Star our <a
							href="https://github.com/candlecorp/wick"
							class="text-primary-600 dark:text-primary-500 hover:underline"
						>
							GitHub Repo
						</a>
						.
					</div>
					<div class="flex justify-center items-center mb-4">
						<Button size="xs" on:click={checkStargazer}>Confirm Star</Button>
					</div>
					<div class="mb-4 text-sm font-light">
						Due to GitHub API limits, it can take up to 10 minutes before the confirmation will
						work.
					</div>
				{/if}
				{#if stargazer_response}
					<Alert color="yellow">
						{stargazer_response_body}
					</Alert>
				{/if}
				<div class="mb-2 text-sm font-light">
					Any questions or comments? Join our <a
						href="https://discord.gg/candle"
						class="text-primary-600 dark:text-primary-500 hover:underline">Discord</a
					>.
				</div>
			</div>
		</Popover>
		{#if token_usage}
			{#if daily_usage < 100}
				<Progressbar
					progress={daily_usage}
					size="h-3"
					labelInside
					animate={true}
					color="green"
					labelInsideClass="bg-blue-600 text-blue-100 text-xs font-medium text-center p-0 leading-none rounded-full"
					class="w-60 my-4"
				/>
			{:else}
				<Progressbar
					progress={daily_usage}
					size="h-3"
					labelInside
					animate={true}
					color="red"
					labelInsideClass="bg-blue-600 text-blue-100 text-xs font-medium text-center p-0 leading-none rounded-full"
					class="w-60 my-4"
				/>
			{/if}
		{/if}
	</div>
	<div class="text-center">
		<iframe
			src="https://ghbtns.com/github-btn.html?user=candlecorp&repo=wick&type=star&count=true"
			frameborder="0"
			scrolling="0"
			width="150"
			height="20"
			title="GitHub"
		/>
	</div>

	{#if loggedIn}
		<Button href="/oidc/logout">Logout</Button>
	{:else}
		<Button href="/login">Login</Button>
	{/if}
</Navbar>
