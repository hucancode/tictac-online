<script>
	import '../styles/app.css';
	import { onMount } from 'svelte';
	import { ensureInitialized } from '$lib/stores/auth.svelte';
	
	// Initialize auth store on client side
	onMount(() => {
		ensureInitialized();
	});
</script>

<slot />