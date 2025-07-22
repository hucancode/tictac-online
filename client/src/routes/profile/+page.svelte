<script lang="ts">
	import { goto } from '$app/navigation';
	import Profile from '../../components/Profile.svelte';
	import { auth } from '$lib/stores/auth.svelte';
	import { onMount } from 'svelte';

	onMount(() => {
		if (!auth.isAuthenticated) {
			goto('/');
		}
	});
</script>

<nav class="bg-gray-800 text-white p-4 mb-8">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="flex items-center justify-between">
			<h1 class="text-2xl font-bold">TicTac Online</h1>
			<div class="flex items-center gap-4">
				<a href="/" class="text-gray-300 hover:text-white">
					← Back to Game
				</a>
			</div>
		</div>
	</div>
</nav>

<div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
	{#if auth.isAuthenticated}
		<div class="bg-white rounded-lg shadow p-6">
			<h2 class="text-2xl font-bold mb-6">My Profile</h2>
			<Profile />
		</div>
	{:else}
		<div class="text-center py-8">
			<p class="text-gray-600">Please log in to view your profile.</p>
			<a href="/" class="text-blue-600 hover:text-blue-800">Go to Login</a>
		</div>
	{/if}
</div>