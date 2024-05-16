<script lang="ts">
	import { getProfile, user, login, logout } from '$lib/auth';
	import { invalidateAll } from '$app/navigation';
	import { onMount } from 'svelte';

	let loading: boolean;
	let username = 'hucancode@gmail.com';
	let password = '';
	onMount(async () => {
		loading = true;
		try {
			await getProfile();
		} catch {
		} finally {
			loading = false;
		}
	});

  async function loginAndRefresh() {
    await login(username, password);
    invalidateAll();
  }

	async function logoutAndRefresh() {
		await logout();
    invalidateAll();
	}
</script>

{#if $user}
  <img alt={$user.email} src={$user.avatar} />
  <button on:click={logoutAndRefresh}>Log out</button>
{:else}
  <input type="text" placeholder="Username" bind:value={username} />
  <input type="password" bind:value={password} />
  <button on:click={loginAndRefresh}>Login</button>
{/if}