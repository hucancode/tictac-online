<script lang="ts">
  import { auth, updateUser } from '$lib/stores/auth.svelte';
  import { userApi } from '$lib/api';
  
  let editingUsername = $state(false);
  let newUsername = $state('');
  let uploadingPicture = $state(false);
  
  $effect(() => {
    if (auth.user) {
      newUsername = auth.user.username;
    }
  });
  
  async function updateUsername() {
    if (!newUsername || newUsername === auth.user?.username) {
      editingUsername = false;
      return;
    }
    
    try {
      const updated = await userApi.updateProfile({ username: newUsername });
      updateUser(updated);
      editingUsername = false;
    } catch (err) {
      console.error('Failed to update username:', err);
      newUsername = auth.user?.username || '';
    }
  }
  
  async function handleFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    
    const file = input.files[0];
    if (!file.type.startsWith('image/')) {
      alert('Please select an image file');
      return;
    }
    
    uploadingPicture = true;
    
    try {
      await userApi.uploadProfilePicture(file);
      // Refresh user data
      const updated = await userApi.getProfile(auth.user!.id);
      updateUser(updated);
    } catch (err) {
      console.error('Failed to upload profile picture:', err);
      alert('Failed to upload profile picture');
    } finally {
      uploadingPicture = false;
    }
  }
</script>

{#if auth.user}
  <div class="bg-white rounded-lg shadow p-6">
    <div class="flex items-center space-x-4 mb-6">
      <div class="relative">
        {#if auth.user.profile_picture}
          <img 
            src={auth.user.profile_picture} 
            alt="Profile" 
            class="w-20 h-20 rounded-full object-cover"
          />
        {:else}
          <div class="w-20 h-20 rounded-full bg-gray-300 flex items-center justify-center">
            <span class="text-2xl text-gray-600">
              {auth.user.username.charAt(0).toUpperCase()}
            </span>
          </div>
        {/if}
        
        <label class="absolute bottom-0 right-0 bg-blue-600 rounded-full p-1 cursor-pointer hover:bg-blue-700">
          <svg class="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"></path>
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 13a3 3 0 11-6 0 3 3 0 016 0z"></path>
          </svg>
          <input 
            type="file" 
            accept="image/*" 
            class="hidden" 
            onchange={handleFileUpload}
            disabled={uploadingPicture}
          />
        </label>
      </div>
      
      <div class="flex-1">
        <div class="flex items-center space-x-2">
          {#if editingUsername}
            <input
              type="text"
              bind:value={newUsername}
              onblur={updateUsername}
              onkeydown={(e) => e.key === 'Enter' && updateUsername()}
              class="text-xl font-bold px-2 py-1 border rounded"
              autofocus
            />
          {:else}
            <h2 class="text-xl font-bold">{auth.user.username}</h2>
            <button
              onclick={() => editingUsername = true}
              class="text-gray-400 hover:text-gray-600"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"></path>
              </svg>
            </button>
          {/if}
        </div>
        <p class="text-gray-600">{auth.user.email}</p>
      </div>
    </div>
    
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div class="text-center">
        <p class="text-2xl font-bold text-blue-600">{auth.user.elo}</p>
        <p class="text-sm text-gray-600">ELO Rating</p>
      </div>
      <div class="text-center">
        <p class="text-2xl font-bold">{auth.user.games_played}</p>
        <p class="text-sm text-gray-600">Games Played</p>
      </div>
      <div class="text-center">
        <p class="text-2xl font-bold">{auth.user.games_won}</p>
        <p class="text-sm text-gray-600">Games Won</p>
      </div>
      <div class="text-center">
        <p class="text-2xl font-bold">{auth.user.win_rate.toFixed(1)}%</p>
        <p class="text-sm text-gray-600">Win Rate</p>
      </div>
    </div>
  </div>
{/if}