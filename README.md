# 🟢 Agar.io Clone

A multiplayer agar.io clone built with **Rust** (Axum, Tokio, SQLite) and vanilla JS + HTML5 Canvas.

## Features

- **Real-time multiplayer** via WebSocket
- **User accounts** with register/login (stored in SQLite)
- **Custom skins** — upload a profile picture that shows on your cell
- **Splitting** (spacebar), **ejecting mass** (W), and **viruses**
- **Leaderboard**, minimap, smooth camera
- **Server-authoritative** — all game logic runs on the server

## Prerequisites

- [Rust](https://rustup.rs/) (1.75+)

## Build & Run

```bash
# Clone/download the project
cd agario-clone

# Build and run (first build takes ~2 min for dependencies)
cargo run --release
```

The server starts at **http://localhost:3000**

## How to Play

1. Open http://localhost:3000 in your browser
2. (Optional) Create an account and upload a skin
3. Enter a name and click **Play**
4. **Mouse** — move your cell
5. **Spacebar** — split
6. **W** — eject mass
7. Eat food and smaller players to grow!

## Project Structure

```
src/
├── main.rs           # Server entry point
├── config.rs         # Game constants
├── server/
│   ├── http.rs       # REST API (auth, skins)
│   └── ws.rs         # WebSocket game handler
├── game/
│   ├── engine.rs     # Game loop & state broadcasting
│   ├── world.rs      # World simulation (tick, collisions)
│   ├── player.rs     # Player/cell structs
│   ├── food.rs       # Food, viruses, ejected mass
│   └── physics.rs    # Collision & distance utilities
├── db/
│   ├── accounts.rs   # Register, login, sessions
│   ├── skins.rs      # Profile picture storage
│   └── schema.rs     # DB table creation
└── protocol/
    └── messages.rs   # Client↔Server JSON messages

static/               # Frontend (served by Axum)
├── index.html
├── game.js           # Canvas renderer + WS client
├── ui.js             # Auth UI, menus
└── style.css
```

## Configuration

Edit `src/config.rs` to tweak game constants:

| Constant | Default | Description |
|----------|---------|-------------|
| `WORLD_SIZE` | 4000 | World dimensions (pixels) |
| `TICK_RATE` | 30 | Server ticks per second |
| `FOOD_COUNT` | 500 | Food pellets on map |
| `STARTING_MASS` | 10 | New player mass |
| `SERVER_PORT` | 3000 | HTTP/WS port |

## Multiplayer

Open multiple browser tabs to http://localhost:3000 — each tab is a separate player. For LAN play, other devices can connect to your machine's IP on port 3000.
