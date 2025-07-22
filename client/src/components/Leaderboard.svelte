<script lang="ts">
  import { onMount } from 'svelte';
  import { leaderboardApi, type LeaderboardEntry } from '$lib/api';
  import { auth } from '$lib/stores/auth.svelte';
  
  let leaderboard = $state<LeaderboardEntry[]>([]);
  let loading = $state(true);
  let currentPage = $state(0);
  const pageSize = 20;
  
  async function loadLeaderboard() {
    loading = true;
    try {
      leaderboard = await leaderboardApi.getLeaderboard(pageSize, currentPage * pageSize);
    } catch (err) {
      console.error('Failed to load leaderboard:', err);
    } finally {
      loading = false;
    }
  }
  
  onMount(() => {
    loadLeaderboard();
  });
  
  function nextPage() {
    currentPage++;
    loadLeaderboard();
  }
  
  function prevPage() {
    if (currentPage > 0) {
      currentPage--;
      loadLeaderboard();
    }
  }
</script>

<div class="bg-white rounded-lg shadow">
  <div class="px-6 py-4 border-b">
    <h2 class="text-xl font-bold">Leaderboard</h2>
  </div>
  
  {#if loading}
    <div class="p-8 text-center">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
    </div>
  {:else if leaderboard.length === 0}
    <div class="p-8 text-center text-gray-500">
      No players yet
    </div>
  {:else}
    <div class="overflow-x-auto">
      <table class="w-full">
        <thead>
          <tr class="text-left text-sm font-medium text-gray-700 border-b">
            <th class="px-6 py-3">Rank</th>
            <th class="px-6 py-3">Player</th>
            <th class="px-6 py-3 text-right">ELO</th>
            <th class="px-6 py-3 text-right">Games</th>
            <th class="px-6 py-3 text-right">Win Rate</th>
          </tr>
        </thead>
        <tbody>
          {#each leaderboard as entry}
            <tr 
              class="border-b hover:bg-gray-50 transition-colors"
              class:bg-blue-50={auth.user?.username === entry.username}
            >
              <td class="px-6 py-4">
                <span class="font-medium text-gray-900">
                  #{entry.rank}
                </span>
              </td>
              <td class="px-6 py-4">
                <div class="flex items-center space-x-3">
                  {#if entry.profile_picture}
                    <img 
                      src={entry.profile_picture} 
                      alt={entry.username}
                      class="w-8 h-8 rounded-full object-cover"
                    />
                  {:else}
                    <div class="w-8 h-8 rounded-full bg-gray-300 flex items-center justify-center">
                      <span class="text-sm text-gray-600">
                        {entry.username.charAt(0).toUpperCase()}
                      </span>
                    </div>
                  {/if}
                  <span class="font-medium">
                    {entry.username}
                    {#if auth.user?.username === entry.username}
                      <span class="text-sm text-blue-600">(You)</span>
                    {/if}
                  </span>
                </div>
              </td>
              <td class="px-6 py-4 text-right font-semibold text-blue-600">
                {entry.elo}
              </td>
              <td class="px-6 py-4 text-right">
                {entry.games_played}
              </td>
              <td class="px-6 py-4 text-right">
                <span class="font-medium">{entry.win_rate.toFixed(1)}%</span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    
    <div class="px-6 py-4 border-t flex justify-between items-center">
      <button
        onclick={prevPage}
        disabled={currentPage === 0}
        class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        Previous
      </button>
      
      <span class="text-sm text-gray-700">
        Page {currentPage + 1}
      </span>
      
      <button
        onclick={nextPage}
        disabled={leaderboard.length < pageSize}
        class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        Next
      </button>
    </div>
  {/if}
</div>