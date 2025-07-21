<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher();

	let roomName = '';
	let showCreateRoom = false;
	let showJoinRoom = false;
	let availableRooms: string[] = [];

	const createRoom = () => {
		if (roomName.trim()) {
			dispatch('createRoom', { roomName: roomName.trim() });
			roomName = '';
			showCreateRoom = false;
		}
	};

	const joinRoom = (room: string) => {
		dispatch('joinRoom', { roomName: room });
		showJoinRoom = false;
	};

	const handleJoinWithName = () => {
		if (roomName.trim()) {
			joinRoom(roomName.trim());
			roomName = '';
		}
	};

	export const setAvailableRooms = (rooms: string[]) => {
		availableRooms = rooms;
	};
</script>

<div class="room-manager">
	{#if !showCreateRoom && !showJoinRoom}
		<div class="flex gap-2">
			<button on:click={() => showCreateRoom = true} class="bg-blue-500 text-white">
				Create Room
			</button>
			<button on:click={() => showJoinRoom = true} class="bg-green-500 text-white">
				Join Room
			</button>
		</div>
	{/if}

	{#if showCreateRoom}
		<div class="room-form">
			<h3 class="text-lg font-semibold mb-2">Create New Room</h3>
			<div class="flex gap-2">
				<input
					type="text"
					bind:value={roomName}
					placeholder="Enter room name"
					class="border px-2 py-1 flex-1"
					on:keydown={(e) => e.key === 'Enter' && createRoom()}
				/>
				<button on:click={createRoom} class="bg-blue-500 text-white">
					Create
				</button>
				<button on:click={() => { showCreateRoom = false; roomName = ''; }} class="bg-gray-500 text-white">
					Cancel
				</button>
			</div>
		</div>
	{/if}

	{#if showJoinRoom}
		<div class="room-form">
			<h3 class="text-lg font-semibold mb-2">Join Room</h3>
			<div class="flex gap-2 mb-4">
				<input
					type="text"
					bind:value={roomName}
					placeholder="Enter room name"
					class="border px-2 py-1 flex-1"
					on:keydown={(e) => e.key === 'Enter' && handleJoinWithName()}
				/>
				<button on:click={handleJoinWithName} class="bg-green-500 text-white">
					Join
				</button>
				<button on:click={() => { showJoinRoom = false; roomName = ''; }} class="bg-gray-500 text-white">
					Cancel
				</button>
			</div>
			
			{#if availableRooms.length > 0}
				<div class="available-rooms">
					<h4 class="font-semibold mb-2">Available Rooms:</h4>
					<div class="flex flex-col gap-2">
						{#each availableRooms as room}
							<button 
								on:click={() => joinRoom(room)} 
								class="border border-gray-300 hover:bg-gray-100 px-4 py-2 text-left"
							>
								{room}
							</button>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.room-manager {
		@apply p-4 border border-gray-300 rounded;
	}
	
	.room-form {
		@apply p-4;
	}
	
	button {
		@apply px-4 py-2 rounded;
	}
</style>