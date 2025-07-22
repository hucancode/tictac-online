<script lang="ts">
  import { login, register, auth } from '$lib/stores/auth.svelte';
  
  interface Props {
    onsuccess?: () => void;
  }
  
  let { onsuccess }: Props = $props();

  let mode = $state<'login' | 'register'>('login');
  let email = $state('');
  let username = $state('');
  let password = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit() {
    error = '';
    
    if (mode === 'register' && password !== confirmPassword) {
      error = 'Passwords do not match';
      return;
    }

    loading = true;

    try {
      if (mode === 'login') {
        await login({ email, password });
      } else {
        await register({ email, username, password });
      }
      onsuccess?.();
    } catch (err: any) {
      error = err.response?.data?.error || `${mode === 'login' ? 'Login' : 'Registration'} failed`;
    } finally {
      loading = false;
    }
  }

  function toggleMode() {
    mode = mode === 'login' ? 'register' : 'login';
    error = '';
  }
</script>

<div class="max-w-md mx-auto mt-8 p-6 bg-white rounded-lg shadow-lg">
  <h2 class="text-2xl font-bold mb-6 text-center">
    {mode === 'login' ? 'Sign In' : 'Create Account'}
  </h2>

  {#if error}
    <div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
      {error}
    </div>
  {/if}

  <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <div class="mb-4">
      <label for="email" class="block text-sm font-medium text-gray-700 mb-2">
        Email
      </label>
      <input
        type="email"
        id="email"
        bind:value={email}
        required
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="you@example.com"
      />
    </div>

    {#if mode === 'register'}
      <div class="mb-4">
        <label for="username" class="block text-sm font-medium text-gray-700 mb-2">
          Username
        </label>
        <input
          type="text"
          id="username"
          bind:value={username}
          required
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="coolplayer123"
        />
      </div>
    {/if}

    <div class="mb-4">
      <label for="password" class="block text-sm font-medium text-gray-700 mb-2">
        Password
      </label>
      <input
        type="password"
        id="password"
        bind:value={password}
        required
        minlength="6"
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="••••••••"
      />
    </div>

    {#if mode === 'register'}
      <div class="mb-6">
        <label for="confirmPassword" class="block text-sm font-medium text-gray-700 mb-2">
          Confirm Password
        </label>
        <input
          type="password"
          id="confirmPassword"
          bind:value={confirmPassword}
          required
          minlength="6"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="••••••••"
        />
      </div>
    {/if}

    <button
      type="submit"
      disabled={loading}
      class="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed"
    >
      {#if loading}
        <span class="inline-block animate-spin rounded-full h-4 w-4 border-b-2 border-white"></span>
      {:else}
        {mode === 'login' ? 'Sign In' : 'Create Account'}
      {/if}
    </button>
  </form>

  <div class="mt-6 text-center">
    <button
      type="button"
      onclick={toggleMode}
      class="text-sm text-blue-600 hover:text-blue-500"
    >
      {mode === 'login' 
        ? "Don't have an account? Sign up" 
        : 'Already have an account? Sign in'}
    </button>
  </div>
</div>