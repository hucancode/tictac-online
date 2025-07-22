# Tic-Tac-Online

A real-time multiplayer Tic-Tac-Toe game with user authentication, ELO ratings, and leaderboards.

## Features

- **Real-time multiplayer gameplay** on a 10x10 board (5-in-a-row to win)
- **ELO rating system** that updates after each game
- **JWT-based authentication** with user registration and login
- **Room-based gameplay** with queue system

## Tech Stack

- **Backend**: Rust with `Axum` web framework
- **Database**: `SurrealDB` for user data, Redis for caching
- **Frontend**: `SvelteKit` with TypeScript and `Tailwind CSS`
- **Real-time**: WebSockets for game communication

## Getting Started

You will need:

- Docker and Docker Compose
- Rust (latest stable)
- Node.js (v22+)
- npm or bun

1. **Start the database services**
   ```bash
   docker-compose up -d
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Run the backend server**
   ```bash
   cd server
   cargo run
   ```
   The server will start on http://localhost:8080

4. **Run the frontend**
   ```bash
   cd client
   npm install
   npm run dev
   ```
   The client will start on http://localhost:5173

## Usage

1. **Register/Login**: Create a new account or login with existing credentials
2. **Join a Room**: Create a new room or join an existing one
3. **Actions**: Inside a room you can **Queue Up**, then room creator will **Start Game**, then **Play**, players take turns placing marks, 5-in-a-row wins!
4. **View Stats**: Check your profile for ELO rating and game statistics
5. **Leaderboard**: See top players ranked by ELO

## Admin Access

Admin users can access the admin dashboard at `/admin` to:
- View system statistics
- Manage user accounts
- Update ELO ratings
- Delete users

To create an admin user, update the `is_admin` field in the database.

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login user
- `GET /api/auth/me` - Get current user profile

### Users
- `GET /api/users/:id` - Get user profile
- `PUT /api/users/profile` - Update profile
- `POST /api/users/profile/picture` - Upload profile picture

### Leaderboard
- `GET /api/leaderboard` - Get leaderboard
- `GET /api/leaderboard/top` - Get top 10 players

### Admin
- `GET /api/admin/users` - List all users
- `PUT /api/admin/users/:id` - Update user
- `DELETE /api/admin/users/:id` - Delete user
- `GET /api/admin/stats` - Get system statistics

### WebSocket
- `ws://localhost:8080/ws/:room?token=<jwt>` - Game room connection

## Development

### Backend Development
```bash
cd server
cargo test         # Run tests
cargo fmt          # Format code
cargo clippy       # Lint code
```

### Frontend Development
```bash
cd client
npm run dev        # Development server
npm run build      # Production build
npm run check      # Type checking
npm run format     # Format code
```

## Docker Deployment

Build and run with Docker:
```bash
docker-compose -f docker-compose.prod.yml up --build
```
