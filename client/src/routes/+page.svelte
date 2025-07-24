<script lang="ts">
	import RoomManager from '../components/RoomManager.svelte';
	import Auth from '../components/Auth.svelte';
	import Leaderboard from '../components/Leaderboard.svelte';
	import { auth, logout } from '$lib/stores/auth.svelte';
	import { getApiUrl } from '$lib/config';
	import { onMount } from 'svelte';
	
	let connected = $state(false);
	let ws = $state<WebSocket | null>(null);
	let log = $state<string[]>([]);
	let currentRoom = $state<string | null>(null);
	let roomManager: RoomManager;
	let showLeaderboard = $state(false);

	const logEvent = (str: string) => {
		log = [...log, str];
	};

	let board = $state<string[][] | null>(null);
	let symbol_pool = ['❌', '⭕'];
	let symbols = $state(new Map<string, string>());
	let turn = $state<string | null>(null);
	let player_id = $state<string | null>(null);
	let chatMessage = $state('');
	
	// New state for updated architecture
	let isRoomCreator = $state(false);
	let roomCreator = $state('');
	let members = $state<string[]>([]);
	let playerQueue = $state<string[]>([]);
	let activePlayers = $state<string[]>([]);
	let myName = $state('');
	let gameResult = $state<{ winner: string, x: number, y: number } | null>(null);
	let returnTimer = $state<number>(5);
	let stayingToReview = $state<boolean>(false);
	
	let isInQueue = $derived(auth.user && playerQueue.includes(auth.user.email));
	let isActivePlayer = $derived(auth.user && activePlayers.includes(auth.user.email));
	let myActivePlayerIndex = $derived(auth.user ? activePlayers.indexOf(auth.user.email) : -1);
	let canMove = $derived(isActivePlayer && myActivePlayerIndex === turn);
	
	// Use authenticated user's email
	let username = $derived(auth.user?.email || '');
	
	const openConnection = (roomName: string) => {
		if (connected || !auth.isAuthenticated) return;
		
		const token = localStorage.getItem('auth_token');
		currentRoom = roomName;
		const wsUrl = getApiUrl().replace('http', 'ws');
		ws = new WebSocket(`${wsUrl}/ws/${roomName}?token=${encodeURIComponent(token || '')}`);
		ws.addEventListener('open', (event) => {
			connected = true;
			logEvent(`Connected to room: ${roomName}`);
			// Register with username
			if (ws && auth.user) {
				ws.send(JSON.stringify({
					type: 'Register',
					name: auth.user.username
				}));
			}
		});
		ws.addEventListener('close', (event) => {
			logEvent(`Disconnected from room: ${currentRoom}`);
			connected = false;
			currentRoom = null;
			board = null;
			isRoomCreator = false;
			roomCreator = '';
			members = [];
			playerQueue = [];
			activePlayers = [];
			gameResult = null;
		});
		ws.addEventListener('error', (event) => {
			logEvent('Connection error');
			console.error(event);
		});
		ws.addEventListener('message', (event) => {
			console.log('Message from server: ', event.data);
			let parsed: any;
			try {
				parsed = JSON.parse(event.data);
			} catch {
				logEvent(`Failed to parse server message: ${event.data}`);
				return;
			}
			switch (parsed.type) {
				case 'JoinedRoom':
					player_id = parsed.your_id.toString();
					isRoomCreator = parsed.is_room_creator;
					roomCreator = parsed.room_creator;
					members = parsed.members;
					playerQueue = parsed.player_queue;
					myName = parsed.members[parsed.your_id];
					logEvent(`Joined room as player ${player_id} (${myName})`);
					break;
				case 'RoomStateUpdate':
					members = parsed.members;
					playerQueue = parsed.player_queue;
					roomCreator = parsed.room_creator;
					break;
				case 'GameStarted':
					logEvent(`Game started with players: ${parsed.players.join(' vs ')}`);
					parsed.players.forEach((player: string, idx: number) => {
						symbols.set(idx.toString(), symbol_pool[idx]);
					});
					activePlayers = parsed.players;
					stayingToReview = false;
					break;
				case 'GameState':
					board = parsed.board;
					turn = parsed.turn;
					break;
				case 'GameEnd':
					gameResult = {
						winner: parsed.winner,
						x: parsed.winner_x,
						y: parsed.winner_y
					};
					logEvent(`Game ended! Winner: ${parsed.winner}`);
					returnTimer = 5;
					
					// Countdown timer
					const countdownInterval = setInterval(() => {
						returnTimer--;
						if (returnTimer <= 0) {
							clearInterval(countdownInterval);
							returnToLobby();
						}
					}, 1000);
					
					// Store interval ID to clear it if user chooses to stay
					(window as any).gameEndCountdown = countdownInterval;
					break;
				case 'Chat':
					logEvent(`${parsed.who}: ${parsed.content}`);
					break;
				default:
					logEvent(`Unknown message type: ${parsed.type}`);
			}
		});
	};

	const returnToLobby = () => {
		// Clear the countdown if it exists
		if ((window as any).gameEndCountdown) {
			clearInterval((window as any).gameEndCountdown);
			delete (window as any).gameEndCountdown;
		}
		
		// Reset game state
		gameResult = null;
		board = null;
		symbols.clear();
		activePlayers = [];
		returnTimer = 5;
		stayingToReview = false;
	};
	
	const stayInGame = () => {
		// Clear the countdown
		if ((window as any).gameEndCountdown) {
			clearInterval((window as any).gameEndCountdown);
			delete (window as any).gameEndCountdown;
		}
		
		// Just clear the popup but keep the board visible
		gameResult = null;
		returnTimer = 5;
		stayingToReview = true;
	};

	const place = (x: number, y: number) => {
		if (!ws || !connected || !canMove) return;
		ws.send(JSON.stringify({ type: 'Place', x, y }));
	};

	const sendChat = () => {
		if (!ws || !connected || !chatMessage.trim()) return;
		ws.send(JSON.stringify({ type: 'Chat', content: chatMessage }));
		chatMessage = '';
	};

	const stepUp = () => {
		if (!ws || !connected) return;
		ws.send(JSON.stringify({ type: 'StepUp' }));
	};

	const stepDown = () => {
		if (!ws || !connected) return;
		ws.send(JSON.stringify({ type: 'StepDown' }));
	};

	const startGame = () => {
		if (!ws || !connected || !isRoomCreator) return;
		ws.send(JSON.stringify({ type: 'StartGame' }));
	};

	const kickMember = (memberId: number) => {
		if (!ws || !connected || !isRoomCreator) return;
		ws.send(JSON.stringify({ type: 'KickMember', member_id: memberId }));
	};

	const leaveRoom = () => {
		if (ws) {
			ws.close();
			ws = null;
		}
	};

	const handleCreateRoom = (event: { roomName: string }) => {
		openConnection(event.roomName);
	};

	const handleJoinRoom = (event: { roomName: string }) => {
		openConnection(event.roomName);
	};
</script>

<svelte:head>
	<title>Tic-Tac-Online</title>
</svelte:head>

<main class="min-h-screen bg-gray-100">
	<nav class="bg-white shadow-sm mb-8">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="flex justify-between h-16">
				<div class="flex items-center">
					<h1 class="text-xl font-bold">Tic-Tac-Online</h1>
				</div>
				<div class="flex items-center space-x-4">
					{#if auth.isAuthenticated}
						<span class="text-gray-700">Welcome, {auth.user?.username}!</span>
						<a
							href="/profile"
							class="text-blue-600 hover:text-blue-800"
						>
							My Profile
						</a>
						<button
							onclick={() => showLeaderboard = !showLeaderboard}
							class="text-blue-600 hover:text-blue-800"
						>
							{showLeaderboard ? 'Hide' : 'Show'} Leaderboard
						</button>
						<button
							onclick={() => logout()}
							class="text-red-600 hover:text-red-800"
						>
							Logout
						</button>
					{/if}
				</div>
			</div>
		</div>
	</nav>

	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		{#if !auth.isAuthenticated}
			<Auth onsuccess={() => {}} />
		{:else}
			{#if showLeaderboard}
				<div class="mb-8">
					<Leaderboard />
				</div>
			{/if}

			{#if !connected}
				<div class="bg-white rounded-lg shadow p-6">
					<h2 class="text-xl font-semibold mb-4">Join or Create a Room</h2>
					<RoomManager 
						bind:this={roomManager}
						oncreateRoom={handleCreateRoom}
						onjoinRoom={handleJoinRoom}
					/>
				</div>
			{:else}
				<div class="bg-white rounded-lg shadow p-6">
					<div class="flex items-center justify-between mb-4">
						<h2 class="text-xl font-semibold">Room: {currentRoom}</h2>
						<div class="flex gap-2">
							{#if !isInQueue && !isActivePlayer}
								<button onclick={stepUp} class="bg-green-500 text-white px-4 py-2 rounded hover:bg-green-600">
									Step Up to Play
								</button>
							{:else if isInQueue && !isActivePlayer}
								<button onclick={stepDown} class="bg-yellow-500 text-white px-4 py-2 rounded hover:bg-yellow-600">
									Step Down
								</button>
							{/if}
							
							{#if isRoomCreator && playerQueue.length >= 2 && activePlayers.length === 0}
								<button onclick={startGame} class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600">
									Start Game
								</button>
							{/if}
							
							<button onclick={leaveRoom} class="bg-red-500 text-white px-4 py-2 rounded hover:bg-red-600">
								Leave Room
							</button>
						</div>
					</div>
					
					<aside class="flex flex-col gap-2 mb-4">
						<div class="flex gap-4">
							<code class="bg-gray-100 px-2 py-1 rounded">You: {auth.user?.username}</code>
							{#if isRoomCreator}
								<code class="bg-purple-100 px-2 py-1 rounded">Room Creator</code>
							{/if}
							{#if isActivePlayer}
								<code class="bg-green-100 px-2 py-1 rounded">Playing: {symbols.get(myActivePlayerIndex.toString())}</code>
								{#if turn !== null && activePlayers.length > 0}
									{#if canMove}
										<code class="bg-green-200 px-2 py-1 font-bold rounded">YOUR TURN</code>
									{:else}
										<code class="bg-yellow-100 px-2 py-1 rounded">Waiting for opponent's move</code>
									{/if}
								{/if}
							{:else if isInQueue}
								<code class="bg-blue-100 px-2 py-1 rounded">In Queue</code>
							{/if}
						</div>
					</aside>
					
					<div class="flex gap-8">
						<div class="flex-1">
							{#if gameResult}
								<div class="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center z-10">
									<div class="bg-white p-8 rounded-lg text-center shadow-xl max-w-md">
										<h2 class="text-3xl font-bold mb-4">Game Over!</h2>
										<p class="text-xl mb-4">
											{#if gameResult.winner === auth.user?.email}
												<span class="text-green-600">🎉 You Won! 🎉</span>
											{:else if gameResult.winner === '' || gameResult.winner === 'Draw'}
												<span class="text-yellow-600">🤝 It's a Draw!</span>
											{:else}
												<span class="text-red-600">😔 You Lost</span>
												<br>
												<span class="text-sm text-gray-600">{gameResult.winner} Won</span>
											{/if}
										</p>
										{#if gameResult.x > 0}
											<p class="text-sm text-gray-600 mb-4">Winning position: ({gameResult.x}, {gameResult.y})</p>
										{/if}
										
										<div class="flex gap-4 justify-center mt-6 mb-4">
											<button
												onclick={stayInGame}
												class="px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
											>
												Stay & Review
											</button>
											<button
												onclick={returnToLobby}
												class="px-6 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600 transition-colors"
											>
												Back to Lobby
											</button>
										</div>
										
										<p class="text-sm text-gray-500">
											Auto-returning to lobby in 
											<span class="font-bold text-lg text-gray-700">{returnTimer}</span> 
											seconds...
										</p>
									</div>
								</div>
							{/if}
							
							{#if board}
								<div class="inline-block">
									{#each board as row, i}
										<div class="flex gap-1">
											{#each row as v, j}
												<button
													disabled={!canMove || v != null || gameResult != null}
													onclick={() => place(i, j)}
													class="w-12 h-12 border-2 border-gray-300 rounded {canMove && v == null && !gameResult ? 'hover:bg-gray-100 cursor-pointer' : 'cursor-not-allowed'} {v != null ? 'bg-gray-50' : ''} {gameResult && i === gameResult.x && j === gameResult.y ? 'bg-yellow-200 ring-2 ring-yellow-400' : ''}"
												>
													{v != null ? symbols.get(v.toString()) : ' '}
												</button>
											{/each}
										</div>
									{/each}
								</div>
								
								{#if stayingToReview}
									<div class="mt-6 text-center">
										<button
											onclick={returnToLobby}
											class="px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
										>
											Back to Lobby
										</button>
										<p class="text-sm text-gray-600 mt-2">Game has ended - reviewing board</p>
									</div>
								{/if}
							{:else if activePlayers.length === 0}
								<div class="text-center py-8 text-gray-500">
									Waiting for game to start...
								</div>
							{/if}
						</div>
						
						<div class="w-96 flex flex-col gap-4">
							<div class="border border-gray-300 rounded p-4 flex-1 flex flex-col max-h-96">
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
										class="flex-1 border rounded px-2 py-1 text-sm"
										onkeydown={(e) => e.key === 'Enter' && sendChat()}
									/>
									<button onclick={sendChat} class="bg-blue-500 text-white text-sm px-4 py-1 rounded hover:bg-blue-600">
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
											{#if isRoomCreator && member !== auth.user?.email}
												<button 
													onclick={() => kickMember(idx)}
													class="text-red-500 hover:bg-red-50 px-2 py-1 text-xs rounded"
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
		{/if}
	</div>
</main>