# Tic-Tac-Online

A real-time multiplayer Tic-Tac-Toe game with user authentication, ELO ratings, and leaderboards.

## Features

- **Real-time multiplayer gameplay** on a 10x10 board (5-in-a-row to win)
- **ELO rating system** that updates after each game
- **Admin dashboard** for user management

## Tech Stack

- **Backend**: Rust with Axum web framework
- **Database**: SurrealDB for all data storage
- **Frontend**: SvelteKit with TypeScript and Tailwind CSS

## Prerequisites

### For Kubernetes deployment (recommended):
- **Minikube** - Local Kubernetes cluster
- **Podman** or **Docker** - Container runtime
- **kubectl** - Kubernetes CLI

### For isolated development (optional):
- **Rust** (1.88+) - For backend development without containers
- **Bun** - For frontend development without containers

## Quick Start

The easiest way to run the entire application:

```bash
# Start everything with one command
make dev
```

This will:
1. Start Minikube (if not running)
2. Build container images
3. Deploy DB, server, client to Kubernetes

The application will be available at:
- **Client**: http://localhost:30030
- **API**: http://localhost:30080
- **Database**: localhost:8000

## Development

### Full Stack Development

All infrastructure is managed through Kubernetes:

```bash
# Main commands
make dev          # Deploy everything
make status       # Check deployment status
make logs-server  # View server logs
make logs-client  # View client logs
make clean        # Remove all resources
make reset        # Clean up and stop Minikube
```

### Isolated Component Development

You can develop individual components locally while keeping the rest of the infrastructure in Kubernetes:

#### Backend Development (Rust)
```bash
make stop-server
cd server
cargo run
```

#### Frontend Development (Bun/Node)
```bash
make stop-client
cd client
bun install       # First time only
bun run dev       # Connects to API at localhost:30080
```

This approach lets you use hot reloading and faster iteration cycles while still leveraging the Kubernetes infrastructure for the database and other services.

## Usage

1. **Register/Login**: Create a new account or login with existing credentials
2. **Join a Room**: Create a new room or join an existing one
3. **Play**: Queue up, wait for game to start, then take turns placing marks

## Admin Access

A default admin user is created on first startup:
- Email: `admin@example.com`
- Password: `adminpass`

Admin users can access `/admin` to manage users and view system statistics.
