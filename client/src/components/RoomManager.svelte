<script lang="ts">
	interface Props {
		oncreateRoom?: (detail: { roomName: string }) => void;
		onjoinRoom?: (detail: { roomName: string }) => void;
	}

	let { oncreateRoom, onjoinRoom }: Props = $props();

	let roomName = $state('');
	let showCreateRoom = $state(false);
	let showJoinRoom = $state(false);
	let availableRooms = $state<string[]>([]);

	function createRoom() {
		if (roomName.trim()) {
			oncreateRoom?.({ roomName: roomName.trim() });
			roomName = '';
			showCreateRoom = false;
		}
	}

	function joinRoom(room: string) {
		onjoinRoom?.({ roomName: room });
		showJoinRoom = false;
	}

	function handleJoinWithName() {
		if (roomName.trim()) {
			joinRoom(roomName.trim());
			roomName = '';
		}
	}

	export function setAvailableRooms(rooms: string[]) {
		availableRooms = rooms;
	}
</script>

<div class="room-manager p-4 border border-gray-300 rounded">
	{#if !showCreateRoom && !showJoinRoom}
		<div class="flex gap-2">
			<button onclick={() => showCreateRoom = true} class="px-4 py-2 rounded bg-blue-500 text-white">
				Create Room
			</button>
			<button onclick={() => showJoinRoom = true} class="px-4 py-2 rounded bg-green-500 text-white">
				Join Room
			</button>
		</div>
	{/if}

	{#if showCreateRoom}
		<div class="room-form p-4">
			<h3 class="text-lg font-semibold mb-2">Create New Room</h3>
			<div class="flex gap-2">
				<input
					type="text"
					bind:value={roomName}
					placeholder="Enter room name"
					class="border px-2 py-1 flex-1"
					onkeydown={(e) => e.key === 'Enter' && createRoom()}
				/>
				<button onclick={createRoom} class="px-4 py-2 rounded bg-blue-500 text-white">
					Create
				</button>
				<button onclick={() => { showCreateRoom = false; roomName = ''; }} class="px-4 py-2 rounded bg-gray-500 text-white">
					Cancel
				</button>
			</div>
		</div>
	{/if}

	{#if showJoinRoom}
		<div class="room-form p-4">
			<h3 class="text-lg font-semibold mb-2">Join Room</h3>
			<div class="flex gap-2 mb-4">
				<input
					type="text"
					bind:value={roomName}
					placeholder="Enter room name"
					class="border px-2 py-1 flex-1"
					onkeydown={(e) => e.key === 'Enter' && handleJoinWithName()}
				/>
				<button onclick={handleJoinWithName} class="px-4 py-2 rounded bg-green-500 text-white">
					Join
				</button>
				<button onclick={() => { showJoinRoom = false; roomName = ''; }} class="px-4 py-2 rounded bg-gray-500 text-white">
					Cancel
				</button>
			</div>
			
			{#if availableRooms.length > 0}
				<div class="available-rooms">
					<h4 class="font-semibold mb-2">Available Rooms:</h4>
					<div class="flex flex-col gap-2">
						{#each availableRooms as room}
							<button 
								onclick={() => joinRoom(room)} 
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