<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { userApi } from '$lib/api';
	import type { UserProfile } from '$lib/api';

	let player = $state<UserProfile | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			const playerId = $page.params.id;
			player = await userApi.getProfile(playerId);
		} catch (err: any) {
			error = err.response?.data?.error || 'Failed to load player profile';
		} finally {
			loading = false;
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
	{#if loading}
		<div class="text-center py-8">
			<p class="text-gray-600">Loading player profile...</p>
		</div>
	{:else if error}
		<div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
			{error}
		</div>
	{:else if player}
		<div class="bg-white rounded-lg shadow p-6">
			<div class="flex items-start gap-6">
				{#if player.profile_picture}
					<img 
						src={player.profile_picture} 
						alt="{player.username}'s profile" 
						class="w-32 h-32 rounded-lg object-cover"
					/>
				{:else}
					<div class="w-32 h-32 bg-gray-200 rounded-lg flex items-center justify-center">
						<span class="text-4xl text-gray-500">👤</span>
					</div>
				{/if}
				
				<div class="flex-1">
					<h2 class="text-3xl font-bold mb-4">{player.username}</h2>
					
					<div class="grid grid-cols-2 md:grid-cols-4 gap-4">
						<div class="bg-gray-50 p-4 rounded-lg text-center">
							<p class="text-sm text-gray-600">ELO Rating</p>
							<p class="text-2xl font-bold text-blue-600">{player.elo}</p>
						</div>
						
						<div class="bg-gray-50 p-4 rounded-lg text-center">
							<p class="text-sm text-gray-600">Games Played</p>
							<p class="text-2xl font-bold">{player.games_played}</p>
						</div>
						
						<div class="bg-gray-50 p-4 rounded-lg text-center">
							<p class="text-sm text-gray-600">Games Won</p>
							<p class="text-2xl font-bold text-green-600">{player.games_won}</p>
						</div>
						
						<div class="bg-gray-50 p-4 rounded-lg text-center">
							<p class="text-sm text-gray-600">Win Rate</p>
							<p class="text-2xl font-bold text-purple-600">{player.win_rate.toFixed(1)}%</p>
						</div>
					</div>
				</div>
			</div>
		</div>
	{/if}
</div>