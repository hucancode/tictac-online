<script>
	let connected = false;
	let ws = null;
	let log = [];

	const logEvent = (str) => {
		log = [...log, str];
	};

	let board = null;
	let symbolPool = ['❌', '⭕'];
	let symbols = {};
	let turn = null;
	let ready = false;
	let player_id = null;
	const openConnection = () => {
		if (connected) return;
		ws = new WebSocket(`ws://127.0.0.1:8080/ws/1`);
		ws.addEventListener('open', (event) => {
			connected = true;
			logEvent('connection open');
		});
		ws.addEventListener('close', (event) => {
			logEvent('connection closed');
			connected = false;
			ready = false;
		});
		ws.addEventListener('message', (event) => {
			console.info(event.data);
			let message = JSON.parse(event.data);
			switch (message.type) {
				case 'JoinedRoom':
					logEvent('You has joined the room with id ' + message.your_id);
					player_id = message.your_id;
					break;
				case 'GameStarted':
					for (let i = 0; i < message.players.length; i++) {
						symbols[message.players[i]] = symbolPool[i];
					}
					break;
				case 'GameState':
					board = message.board;
					turn = message.turn;
					break;
				case 'GameEnd':
					ready = false;
					logEvent('game ended, winning move is at ' + message.winner_x + '-' + message.winner_y);
					break;
				case 'Chat':
					logEvent(message.who + ': ' + message.content);
					break;
			}
		});
	};
	const toggleReady = () => {
		ready = !ready;
		if (ws) ws.send(JSON.stringify({ type: 'ReadyVote', accept: ready }));
	};
	const place = (x, y) => {
		if (ws) ws.send(JSON.stringify({ type: 'Place', x, y }));
	};
</script>

<main class="w-full h-full p-10 flex flex-col gap-2">
	<div class="container mx-auto flex gap-2">
		{#if !connected}
			<button on:click={openConnection}>Connect</button>
		{:else}
			<button on:click={toggleReady}>Ready {ready ? '✅' : ''}</button>
		{/if}
	</div>
	<aside class="container mx-auto flex gap-2">
		{#if player_id != null && turn != null}
			<code>You are: {symbols[player_id]}</code>
			<code>Turn of: {symbols[turn]}</code>
		{/if}
	</aside>
	<div class="container mx-auto flex flex-col gap-2">
		{#if board}
			{#each board as row, i}
				<div class="flex gap-2">
					{#each row as v, j}
						<button
							enabled={player_id == turn}
							on:click={() => place(i, j)}
							class="w-12 h-12 border"
						>
							{v != null ? symbols[v] : ' '}
						</button>
					{/each}
				</div>
			{/each}
		{/if}
	</div>
	<ul>
		{#each log as event}
			<li>{event}</li>
		{/each}
	</ul>
</main>

<style>
	main {
		font-family: sans-serif;
	}
	button {
		@apply border border-gray-300 px-4 py-1;
	}
</style>
