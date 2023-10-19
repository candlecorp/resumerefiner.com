<script lang="ts">
</script>

<div class="mt-2">
	<div class="skeleton skeleton-text" />
	<div class="skeleton skeleton-text" />
	<div class="skeleton skeleton-text skeleton-text-lastline" />
</div>

<style>
	.skeleton {
		animation: skeleton-loading 1s linear infinite alternate;
	}

	.skeleton-text {
		width: 100%;
		height: 0.7rem;
		margin-bottom: 0.5rem;
		border-radius: 0.25rem;
	}

	.skeleton-text-lastline {
		width: 75%;
	}

	@keyframes skeleton-loading {
		0% {
			background-color: hsl(200, 20%, 80%);
		}
		100% {
			background-color: hsl(200, 20%, 95%);
		}
	}
</style>
