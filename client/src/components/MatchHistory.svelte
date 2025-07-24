<script lang="ts">
  import { onMount } from 'svelte';
  import { gamesApi, type MatchHistoryItem, type MatchHistoryResponse } from '$lib/api';
  
  let matchHistory: MatchHistoryResponse | null = $state(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let currentPage = $state(1);
  
  async function loadMatchHistory(page: number = 1) {
    loading = true;
    error = null;
    
    try {
      matchHistory = await gamesApi.getMatchHistory(page, 20);
      currentPage = page;
    } catch (err: any) {
      error = err.response?.data?.error || 'Failed to load match history';
    } finally {
      loading = false;
    }
  }
  
  onMount(() => {
    loadMatchHistory();
  });
  
  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  }
  
  function getResultClass(result: string): string {
    switch (result) {
      case 'win': return 'text-green-600';
      case 'loss': return 'text-red-600';
      case 'draw': return 'text-yellow-600';
      default: return '';
    }
  }
  
  function getEloChangeClass(change: number): string {
    if (change > 0) return 'text-green-600';
    if (change < 0) return 'text-red-600';
    return 'text-gray-600';
  }
</script>

<div class="bg-white rounded-lg shadow-md p-6">
  <h2 class="text-2xl font-bold mb-4">Match History</h2>
  
  {#if loading}
    <div class="text-center py-8">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
    </div>
  {:else if error}
    <div class="text-red-600 text-center py-4">
      {error}
    </div>
  {:else if matchHistory && matchHistory.matches.length > 0}
    <div class="overflow-x-auto">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Date
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Opponent
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Result
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              ELO Change
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Your ELO
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Opponent ELO
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each matchHistory.matches as match}
            <tr class="hover:bg-gray-50">
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {formatDate(match.ended_at || match.created_at)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {match.opponent_name}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium {getResultClass(match.result)}">
                {match.result.toUpperCase()}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium {getEloChangeClass(match.elo_change)}">
                {match.elo_change > 0 ? '+' : ''}{match.elo_change}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {match.my_elo_before} → {match.my_elo_after}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {match.opponent_elo_before} → {match.opponent_elo_after}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    
    {#if matchHistory.total > matchHistory.limit}
      <div class="mt-4 flex justify-between items-center">
        <div class="text-sm text-gray-700">
          Showing {((currentPage - 1) * matchHistory.limit) + 1} to {Math.min(currentPage * matchHistory.limit, matchHistory.total)} of {matchHistory.total} matches
        </div>
        <div class="flex space-x-2">
          <button
            onclick={() => loadMatchHistory(currentPage - 1)}
            disabled={currentPage === 1}
            class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Previous
          </button>
          <button
            onclick={() => loadMatchHistory(currentPage + 1)}
            disabled={currentPage * matchHistory.limit >= matchHistory.total}
            class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Next
          </button>
        </div>
      </div>
    {/if}
  {:else}
    <p class="text-gray-500 text-center py-8">No matches played yet.</p>
  {/if}
</div>