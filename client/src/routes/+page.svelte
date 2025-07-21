<script lang="ts">
	import RoomManager from '../components/RoomManager.svelte';
	
	let connected: boolean = false;
	let ws: WebSocket | null = null;
	let log: string[] = [];
	let currentRoom: string | null = null;
	let roomManager: RoomManager;

	const logEvent = (str: string) => {
		log = [...log, str];
	};

	let board: string[][] | null = null;
	let symbol_pool = ['❌', '⭕'];
	let symbols: Map<string, string> = new Map();
	let turn: string | null = null;
	let player_id: string | null = null;
	let chatMessage = '';
	
	// New state for updated architecture
	let isRoomCreator = false;
	let roomCreator = '';
	let members: string[] = [];
	let playerQueue: string[] = [];
	let activePlayers: string[] = [];
	let myName = '';
	let gameResult: { winner: string, x: number, y: number } | null = null;
	
	$: isInQueue = playerQueue.includes(myName);
	$: isActivePlayer = activePlayers.includes(myName);
	$: myActivePlayerIndex = activePlayers.indexOf(myName);
	$: canMove = isActivePlayer && myActivePlayerIndex === turn;
	
	// Generate a fake user ID on component mount  
	let username = `User_${Math.random().toString(36).substring(2, 9)}`;
	
	const openConnection = (roomName: string) => {
		if (connected) return;
		currentRoom = roomName;
		ws = new WebSocket(`ws://127.0.0.1:8080/ws/${roomName}?user=${encodeURIComponent(username)}`);
		ws.addEventListener('open', (event) => {
			connected = true;
			logEvent(`Connected to room: ${roomName}`);
		});
		ws.addEventListener('close', (event) => {
			logEvent(`Disconnected from room: ${currentRoom}`);
			connected = false;
			currentRoom = null;
			board = null;
			symbols.clear();
			turn = null;
			player_id = null;
			isRoomCreator = false;
			roomCreator = '';
			members = [];
			playerQueue = [];
			activePlayers = [];
			myName = '';
			gameResult = null;
		});
		ws.addEventListener('message', (event) => {
			console.info(event.data);
			let message = JSON.parse(event.data);
			switch (message.type) {
				case 'JoinedRoom':
					player_id = message.your_id;
					isRoomCreator = message.is_room_creator;
					roomCreator = message.room_creator;
					members = message.members;
					playerQueue = message.player_queue;
					myName = members[player_id] || 'Player ' + (player_id + 1);
					
					if (isRoomCreator) {
						logEvent('You have created and joined the room');
					} else {
						logEvent('You have joined the room');
					}
					break;
				case 'GameStarted':
					activePlayers = message.players;
					for (let i = 0; i < message.players.length; i++) {
						let playerName = message.players[i];
						let playerIndex = members.indexOf(playerName);
						symbols.set(playerIndex.toString(), symbol_pool[i]);
					}
					logEvent('Game started between ' + message.players.join(' and '));
					break;
				case 'RoomStateUpdate':
					members = message.members;
					playerQueue = message.player_queue;
					roomCreator = message.room_creator;
					// Update myName in case members list changed
					if (player_id !== null && player_id < members.length) {
						myName = members[player_id];
						isRoomCreator = myName === roomCreator;
					}
					break;
				case 'GameState':
					board = message.board;
					turn = message.turn;
					// Log whose turn it is
					if (turn !== null && turn < activePlayers.length) {
						logEvent(`It's ${activePlayers[turn]}'s turn`);
					}
					break;
				case 'GameEnd':
					gameResult = {
						winner: message.winner,
						x: message.winner_x,
						y: message.winner_y
					};
					logEvent(`Game ended! Winner: ${message.winner}${message.winner_x > 0 ? ` at position ${message.winner_x + 1}-${message.winner_y + 1}` : ''}`);
					// Clear game state after a delay to show result
					setTimeout(() => {
						gameResult = null;
						activePlayers = [];
						board = null;
						turn = null;
						symbols.clear();
					}, 5000); // Show result for 5 seconds
					break;
				case 'Chat':
					logEvent(message.who + ': ' + message.content);
					break;
			}
		});
	};
	
	const handleCreateRoom = (event: CustomEvent) => {
		openConnection(event.detail.roomName);
	};
	
	const handleJoinRoom = (event: CustomEvent) => {
		openConnection(event.detail.roomName);
	};
	
	const leaveRoom = () => {
		if (ws) {
			ws.close();
			ws = null;
		}
	};
	
	const sendChat = () => {
		if (ws && chatMessage.trim()) {
			ws.send(JSON.stringify({ type: 'Chat', content: chatMessage.trim() }));
			chatMessage = '';
		}
	};
	const stepUp = () => {
		if (ws) ws.send(JSON.stringify({ type: 'StepUp' }));
	};
	
	const stepDown = () => {
		if (ws) ws.send(JSON.stringify({ type: 'StepDown' }));
	};
	
	const startGame = () => {
		if (ws) ws.send(JSON.stringify({ type: 'StartGame' }));
	};
	
	const kickMember = (memberId: number) => {
		if (ws) ws.send(JSON.stringify({ type: 'KickMember', member_id: memberId }));
	};
	const place = (x: number, y: number) => {
		if (ws) ws.send(JSON.stringify({ type: 'Place', x, y }));
	};
</script>

<main class="w-full h-full p-10 flex flex-col gap-4">
	<h1 class="text-3xl font-bold">Tic-Tac Online</h1>
	
	{#if !connected}
		<div class="space-y-4">
			<div class="p-4 border border-gray-300 rounded">
				<h3 class="text-lg font-semibold mb-2">Set Your Name</h3>
				<input
					type="text"
					bind:value={username}
					placeholder="Enter your username"
					class="w-full border px-3 py-2 rounded"
				/>
			</div>
			
			<RoomManager 
				bind:this={roomManager}
				on:createRoom={handleCreateRoom}
				on:joinRoom={handleJoinRoom}
			/>
		</div>
	{:else}
		<div class="container mx-auto">
			<div class="flex items-center justify-between mb-4">
				<h2 class="text-xl font-semibold">Room: {currentRoom}</h2>
				<div class="flex gap-2">
					{#if !isInQueue && !isActivePlayer}
						<button on:click={stepUp} class="bg-green-500 text-white">
							Step Up to Play
						</button>
					{:else if isInQueue && !isActivePlayer}
						<button on:click={stepDown} class="bg-yellow-500 text-white">
							Step Down
						</button>
					{/if}
					
					{#if isRoomCreator && playerQueue.length >= 2 && activePlayers.length === 0}
						<button on:click={startGame} class="bg-blue-500 text-white">
							Start Game
						</button>
					{/if}
					
					<button on:click={leaveRoom} class="bg-red-500 text-white">
						Leave Room
					</button>
				</div>
			</div>
			
			<aside class="flex flex-col gap-2 mb-4">
				<div class="flex gap-4">
					<code class="bg-gray-100 px-2 py-1">You: {myName}</code>
					{#if isRoomCreator}
						<code class="bg-purple-100 px-2 py-1">Room Creator</code>
					{/if}
					{#if isActivePlayer}
						<code class="bg-green-100 px-2 py-1">Playing: {symbols.get(player_id?.toString())}</code>
						{#if turn !== null && activePlayers.length > 0}
							{#if canMove}
								<code class="bg-green-200 px-2 py-1 font-bold">YOUR TURN</code>
							{:else}
								<code class="bg-yellow-100 px-2 py-1">Waiting for {activePlayers[turn]}'s move</code>
							{/if}
						{/if}
					{:else if isInQueue}
						<code class="bg-blue-100 px-2 py-1">In Queue</code>
					{:else}
						<code class="bg-gray-200 px-2 py-1">Spectating</code>
					{/if}
				</div>
				
				{#if playerQueue.length > 0}
					<div class="text-sm">
						<span class="font-semibold">Player Queue:</span> {playerQueue.join(', ')}
					</div>
				{/if}
			</aside>
			
			<div class="flex gap-8">
				<div class="flex flex-col gap-2 relative">
					{#if gameResult}
						<div class="absolute inset-0 bg-black bg-opacity-75 flex items-center justify-center z-10 rounded">
							<div class="bg-white p-8 rounded-lg text-center">
								<h2 class="text-3xl font-bold mb-4">Game Over!</h2>
								<p class="text-xl mb-4">
									{#if gameResult.winner === myName}
										<span class="text-green-600">🎉 You Won! 🎉</span>
									{:else}
										<span class="text-red-600">{gameResult.winner} Won</span>
									{/if}
								</p>
								{#if gameResult.x > 0}
									<p class="text-sm text-gray-600">Winning position: {gameResult.x + 1}-{gameResult.y + 1}</p>
								{/if}
								<p class="text-sm text-gray-500 mt-4">Returning to lobby in 5 seconds...</p>
							</div>
						</div>
					{/if}
					
					{#if board}
						{#each board as row, i}
							<div class="flex gap-2">
								{#each row as v, j}
									<button
										disabled={!canMove || v != null || gameResult != null}
										on:click={() => place(i, j)}
										class="w-12 h-12 border {canMove && v == null && !gameResult ? 'hover:bg-gray-100 cursor-pointer' : 'cursor-not-allowed'} {v != null ? 'bg-gray-50' : ''} {gameResult && i === gameResult.x && j === gameResult.y ? 'bg-yellow-200 ring-2 ring-yellow-400' : ''}"
									>
										{v != null ? symbols.get(v.toString()) : ' '}
									</button>
								{/each}
							</div>
						{/each}
					{/if}
				</div>
				
				<div class="flex-1 flex flex-col gap-4">
					<div class="border border-gray-300 rounded p-4 flex-1 flex flex-col">
						<h3 class="font-semibold mb-2">Chat & Events</h3>
						<div class="flex-1 overflow-y-auto mb-2 border-t pt-2">
							<ul class="text-sm">
								{#each log as event}
									<li class="mb-1">{event}</li>
								{/each}
							</ul>
						</div>
						<div class="flex gap-2 border-t pt-2">
							<input
								type="text"
								bind:value={chatMessage}
								placeholder="Type a message..."
								class="flex-1 border px-2 py-1 text-sm"
								on:keydown={(e) => e.key === 'Enter' && sendChat()}
							/>
							<button on:click={sendChat} class="bg-blue-500 text-white text-sm px-4">
								Send
							</button>
						</div>
					</div>
					
					<div class="border border-gray-300 rounded p-4">
						<h3 class="font-semibold mb-2">Room Members ({members.length})</h3>
						<ul class="text-sm">
							{#each members as member, idx}
								<li class="flex justify-between items-center py-1">
									<span>
										{member}
										{#if member === roomCreator}
											<span class="text-purple-600 font-semibold">(Creator)</span>
										{/if}
										{#if activePlayers.includes(member)}
											<span class="text-green-600">(Playing)</span>
										{:else if playerQueue.includes(member)}
											<span class="text-blue-600">(In Queue)</span>
										{/if}
									</span>
									{#if isRoomCreator && idx !== player_id}
										<button 
											on:click={() => kickMember(idx)}
											class="text-red-500 hover:bg-red-50 px-2 py-1 text-xs"
										>
											Kick
										</button>
									{/if}
								</li>
							{/each}
						</ul>
					</div>
				</div>
			</div>
		</div>
	{/if}
</main>

<style>
	main {
		font-family: sans-serif;
	}
	button {
		@apply border border-gray-300 px-4 py-1;
	}
</style>
