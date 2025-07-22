<script lang="ts">
  import { auth } from '$lib/stores/auth.svelte';
  import { adminApi, type UserProfile } from '$lib/api';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  
  let users = $state<UserProfile[]>([]);
  let stats = $state<any>(null);
  let loading = $state(true);
  let searchTerm = $state('');
  let editingUser = $state<UserProfile | null>(null);
  let editForm = $state({
    email: '',
    username: '',
    elo: 0,
    is_admin: false
  });
  
  onMount(async () => {
    if (!auth.isAuthenticated || !auth.isAdmin) {
      goto('/');
      return;
    }
    
    await loadData();
  });
  
  async function loadData() {
    loading = true;
    try {
      const [usersData, statsData] = await Promise.all([
        adminApi.listUsers(),
        adminApi.getStats()
      ]);
      users = usersData;
      stats = statsData;
    } catch (err) {
      console.error('Failed to load admin data:', err);
    } finally {
      loading = false;
    }
  }
  
  async function searchUsers() {
    loading = true;
    try {
      users = await adminApi.listUsers(50, 0, searchTerm);
    } catch (err) {
      console.error('Failed to search users:', err);
    } finally {
      loading = false;
    }
  }
  
  function startEdit(user: UserProfile) {
    editingUser = user;
    editForm = {
      email: user.email,
      username: user.username,
      elo: user.elo,
      is_admin: false
    };
  }
  
  async function saveEdit() {
    if (!editingUser) return;
    
    try {
      const updates: any = {};
      if (editForm.email !== editingUser.email) updates.email = editForm.email;
      if (editForm.username !== editingUser.username) updates.username = editForm.username;
      if (editForm.elo !== editingUser.elo) updates.elo = editForm.elo;
      if (editForm.is_admin !== undefined) updates.is_admin = editForm.is_admin;
      
      await adminApi.updateUser(editingUser.id, updates);
      editingUser = null;
      await loadData();
    } catch (err) {
      console.error('Failed to update user:', err);
      alert('Failed to update user');
    }
  }
  
  async function deleteUser(userId: string) {
    if (!confirm('Are you sure you want to delete this user?')) return;
    
    try {
      await adminApi.deleteUser(userId);
      await loadData();
    } catch (err) {
      console.error('Failed to delete user:', err);
      alert('Failed to delete user');
    }
  }
</script>

<svelte:head>
  <title>Admin Dashboard - Tic-Tac-Online</title>
</svelte:head>

{#if !auth.isAdmin}
  <div class="min-h-screen flex items-center justify-center">
    <p class="text-xl text-gray-600">Access denied. Admin privileges required.</p>
  </div>
{:else}
  <div class="min-h-screen bg-gray-100">
    <nav class="bg-white shadow-sm mb-8">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between h-16">
          <div class="flex items-center">
            <h1 class="text-xl font-bold">Admin Dashboard</h1>
          </div>
          <div class="flex items-center">
            <a href="/" class="text-blue-600 hover:text-blue-800">
              Back to Game
            </a>
          </div>
        </div>
      </div>
    </nav>
    
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      {#if loading && !stats}
        <div class="text-center py-8">
          <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        </div>
      {:else}
        <!-- Stats Cards -->
        {#if stats}
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
            <div class="bg-white rounded-lg shadow p-6">
              <h3 class="text-sm font-medium text-gray-500">Total Users</h3>
              <p class="text-3xl font-bold text-gray-900">{stats.total_users || 0}</p>
            </div>
            <div class="bg-white rounded-lg shadow p-6">
              <h3 class="text-sm font-medium text-gray-500">Total Games</h3>
              <p class="text-3xl font-bold text-gray-900">{stats.total_games || 0}</p>
            </div>
            <div class="bg-white rounded-lg shadow p-6">
              <h3 class="text-sm font-medium text-gray-500">Average ELO</h3>
              <p class="text-3xl font-bold text-gray-900">{Math.round(stats.average_elo || 1200)}</p>
            </div>
          </div>
        {/if}
        
        <!-- User Management -->
        <div class="bg-white rounded-lg shadow">
          <div class="px-6 py-4 border-b">
            <div class="flex justify-between items-center">
              <h2 class="text-xl font-bold">User Management</h2>
              <div class="flex gap-2">
                <input
                  type="text"
                  bind:value={searchTerm}
                  placeholder="Search users..."
                  class="px-3 py-2 border rounded-md"
                  onkeydown={(e) => e.key === 'Enter' && searchUsers()}
                />
                <button
                  onclick={searchUsers}
                  class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
                >
                  Search
                </button>
              </div>
            </div>
          </div>
          
          <div class="overflow-x-auto">
            <table class="w-full">
              <thead>
                <tr class="text-left text-sm font-medium text-gray-700 border-b">
                  <th class="px-6 py-3">User</th>
                  <th class="px-6 py-3">Email</th>
                  <th class="px-6 py-3 text-right">ELO</th>
                  <th class="px-6 py-3 text-right">Games</th>
                  <th class="px-6 py-3 text-right">Win Rate</th>
                  <th class="px-6 py-3 text-center">Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each users as user}
                  <tr class="border-b hover:bg-gray-50">
                    <td class="px-6 py-4">
                      <div class="flex items-center space-x-3">
                        {#if user.profile_picture}
                          <img 
                            src={user.profile_picture} 
                            alt={user.username}
                            class="w-8 h-8 rounded-full object-cover"
                          />
                        {:else}
                          <div class="w-8 h-8 rounded-full bg-gray-300 flex items-center justify-center">
                            <span class="text-sm text-gray-600">
                              {user.username.charAt(0).toUpperCase()}
                            </span>
                          </div>
                        {/if}
                        <span class="font-medium">{user.username}</span>
                      </div>
                    </td>
                    <td class="px-6 py-4 text-gray-600">{user.email}</td>
                    <td class="px-6 py-4 text-right font-semibold">{user.elo}</td>
                    <td class="px-6 py-4 text-right">{user.games_played}</td>
                    <td class="px-6 py-4 text-right">{user.win_rate.toFixed(1)}%</td>
                    <td class="px-6 py-4">
                      <div class="flex justify-center gap-2">
                        <button
                          onclick={() => startEdit(user)}
                          class="text-blue-600 hover:text-blue-800"
                        >
                          Edit
                        </button>
                        <button
                          onclick={() => deleteUser(user.id)}
                          class="text-red-600 hover:text-red-800"
                        >
                          Delete
                        </button>
                      </div>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {/if}
    </div>
    
    <!-- Edit Modal -->
    {#if editingUser}
      <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div class="bg-white rounded-lg p-6 w-full max-w-md">
          <h3 class="text-xl font-bold mb-4">Edit User</h3>
          
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Email</label>
              <input
                type="email"
                bind:value={editForm.email}
                class="w-full px-3 py-2 border rounded-md"
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Username</label>
              <input
                type="text"
                bind:value={editForm.username}
                class="w-full px-3 py-2 border rounded-md"
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">ELO Rating</label>
              <input
                type="number"
                bind:value={editForm.elo}
                class="w-full px-3 py-2 border rounded-md"
              />
            </div>
            
            <div>
              <label class="flex items-center">
                <input
                  type="checkbox"
                  bind:checked={editForm.is_admin}
                  class="mr-2"
                />
                <span class="text-sm font-medium text-gray-700">Admin User</span>
              </label>
            </div>
          </div>
          
          <div class="flex justify-end gap-2 mt-6">
            <button
              onclick={() => editingUser = null}
              class="px-4 py-2 text-gray-700 bg-gray-200 rounded hover:bg-gray-300"
            >
              Cancel
            </button>
            <button
              onclick={saveEdit}
              class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              Save
            </button>
          </div>
        </div>
      </div>
    {/if}
  </div>
{/if}