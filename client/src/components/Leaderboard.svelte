<script lang="ts">
  import { onMount } from 'svelte';
  import { leaderboardApi, type LeaderboardResponse } from '$lib/api';
  import { auth } from '$lib/stores/auth.svelte';
  
  let leaderboardData = $state<LeaderboardResponse | null>(null);
  let loading = $state(true);
  let currentPage = $state(1);
  let playerRank = $state<number | null>(null);
  
  async function loadLeaderboard(page: number = 1) {
    loading = true;
    try {
      leaderboardData = await leaderboardApi.getLeaderboard(page, 20);
      currentPage = page;
      
      // Also check if current user has a rank
      if (auth.user?.id) {
        try {
          const rankData = await leaderboardApi.getPlayerRank(auth.user.id);
          playerRank = rankData.rank;
        } catch (err) {
          console.error('Failed to get player rank:', err);
        }
      }
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
    if (leaderboardData && currentPage < leaderboardData.total_pages) {
      loadLeaderboard(currentPage + 1);
    }
  }
  
  function prevPage() {
    if (currentPage > 1) {
      loadLeaderboard(currentPage - 1);
    }
  }
  
  function goToPage(page: number) {
    loadLeaderboard(page);
  }
  
  // Generate page numbers to show
  function getPageNumbers(): number[] {
    if (!leaderboardData) return [];
    
    const totalPages = leaderboardData.total_pages;
    const pages: number[] = [];
    
    // Always show first page
    pages.push(1);
    
    // Show pages around current page
    const start = Math.max(2, currentPage - 2);
    const end = Math.min(totalPages - 1, currentPage + 2);
    
    if (start > 2) pages.push(-1); // Ellipsis
    
    for (let i = start; i <= end; i++) {
      pages.push(i);
    }
    
    if (end < totalPages - 1) pages.push(-1); // Ellipsis
    
    // Always show last page if there is more than one page
    if (totalPages > 1) pages.push(totalPages);
    
    return pages;
  }
</script>

<div class="bg-white rounded-lg shadow">
  <div class="px-6 py-4 border-b">
    <div class="flex justify-between items-center">
      <h2 class="text-xl font-bold">Leaderboard</h2>
      <span class="text-sm text-gray-500">Top 1000 Players</span>
    </div>
    {#if playerRank && auth.user}
      <div class="mt-2 text-sm text-gray-600">
        Your rank: <span class="font-bold text-blue-600">#{playerRank}</span>
      </div>
    {:else if auth.user && !playerRank}
      <div class="mt-2 text-sm text-gray-500">
        You are not ranked in the top 1000
      </div>
    {/if}
  </div>
  
  {#if loading}
    <div class="p-8 text-center">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
    </div>
  {:else if !leaderboardData || leaderboardData.entries.length === 0}
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
          {#each leaderboardData.entries as entry}
            <tr 
              class="border-b hover:bg-gray-50 transition-colors"
              class:bg-blue-50={auth.user?.id === entry.user_id}
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
                  <a 
                    href="/player/{entry.user_id}"
                    class="font-medium hover:text-blue-600 hover:underline"
                  >
                    {entry.username}
                    {#if auth.user?.id === entry.user_id}
                      <span class="text-sm text-blue-600">(You)</span>
                    {/if}
                  </a>
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
    
    {#if leaderboardData.total_pages > 1}
      <div class="px-6 py-4 border-t">
        <div class="flex flex-col sm:flex-row justify-between items-center gap-4">
          <div class="text-sm text-gray-700">
            Showing {((currentPage - 1) * leaderboardData.limit) + 1} to {Math.min(currentPage * leaderboardData.limit, leaderboardData.total)} of {leaderboardData.total} players
          </div>
          
          <nav class="flex items-center space-x-1">
            <button
              onclick={prevPage}
              disabled={currentPage === 1}
              class="px-3 py-1 text-sm font-medium text-gray-700 bg-white border rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Previous
            </button>
            
            {#each getPageNumbers() as pageNum}
              {#if pageNum === -1}
                <span class="px-2 text-gray-500">...</span>
              {:else}
                <button
                  onclick={() => goToPage(pageNum)}
                  class="px-3 py-1 text-sm font-medium rounded-md {pageNum === currentPage ? 'bg-blue-600 text-white' : 'text-gray-700 bg-white border hover:bg-gray-50'}"
                >
                  {pageNum}
                </button>
              {/if}
            {/each}
            
            <button
              onclick={nextPage}
              disabled={currentPage === leaderboardData.total_pages}
              class="px-3 py-1 text-sm font-medium text-gray-700 bg-white border rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Next
            </button>
          </nav>
        </div>
      </div>
    {/if}
  {/if}
</div>