// ── Game Client ──

const Game = {
    canvas: null,
    ctx: null,
    ws: null,
    playerId: null,
    worldSize: 4000,

    // Game state from server
    players: [],
    food: [],
    viruses: [],
    leaderboard: [],

    // Previous state for interpolation
    prevPlayers: [],
    interpFactor: 0,
    lastStateTime: 0,

    // Camera
    camera: { x: 0, y: 0, scale: 1, targetX: 0, targetY: 0, targetScale: 1 },

    // Input
    mouse: { x: 0, y: 0, worldX: 0, worldY: 0 },

    // Skin cache
    skinImages: {},

    // Animation
    animFrame: null,
    running: false,

    init() {
        this.canvas = document.getElementById('gameCanvas');
        this.ctx = this.canvas.getContext('2d');
        this.resize();
        window.addEventListener('resize', () => this.resize());

        // Mouse input
        this.canvas.addEventListener('mousemove', (e) => {
            this.mouse.x = e.clientX;
            this.mouse.y = e.clientY;
            this.updateMouseWorld();
            this.sendMove();
        });

        // Touch input
        this.canvas.addEventListener('touchmove', (e) => {
            e.preventDefault();
            const touch = e.touches[0];
            this.mouse.x = touch.clientX;
            this.mouse.y = touch.clientY;
            this.updateMouseWorld();
            this.sendMove();
        }, { passive: false });

        // Keyboard
        window.addEventListener('keydown', (e) => {
            if (e.code === 'Space') {
                e.preventDefault();
                this.sendSplit();
            } else if (e.code === 'KeyW') {
                this.sendEject();
            }
        });
    },

    resize() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
    },

    connect(name, token) {
        if (this.ws) {
            this.ws.close();
        }

        const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
        this.ws = new WebSocket(`${protocol}//${location.host}/ws`);

        this.ws.onopen = () => {
            this.ws.send(JSON.stringify({
                type: 'join',
                name: name,
                token: token || null,
            }));
        };

        this.ws.onmessage = (event) => {
            const msg = JSON.parse(event.data);
            this.handleMessage(msg);
        };

        this.ws.onclose = () => {
            this.running = false;
            if (this.animFrame) {
                cancelAnimationFrame(this.animFrame);
            }
        };

        this.ws.onerror = () => {
            console.error('WebSocket error');
        };
    },

    handleMessage(msg) {
        switch (msg.type) {
            case 'joined':
                this.playerId = msg.id;
                this.worldSize = msg.world_size;
                this.running = true;
                this.gameLoop();
                break;

            case 'state':
                // Store previous state for interpolation
                this.prevPlayers = this.players;
                this.players = msg.players;
                this.food = msg.food;
                this.viruses = msg.viruses;
                this.leaderboard = msg.leaderboard;
                this.lastStateTime = performance.now();
                this.interpFactor = 0;

                // Update camera target
                const me = this.players.find(p => p.id === this.playerId);
                if (me && me.cells.length > 0) {
                    let totalMass = 0;
                    let cx = 0, cy = 0;
                    for (const cell of me.cells) {
                        const mass = (cell.radius / 4) ** 2;
                        cx += cell.x * mass;
                        cy += cell.y * mass;
                        totalMass += mass;
                    }
                    this.camera.targetX = cx / totalMass;
                    this.camera.targetY = cy / totalMass;
                    const viewScale = Math.sqrt(totalMass / 10);
                    this.camera.targetScale = Math.max(1, viewScale);

                    UI.updateScore(Math.floor(totalMass));
                }

                // Update leaderboard
                UI.updateLeaderboard(this.leaderboard, this.playerId);

                // Preload skins
                for (const p of this.players) {
                    if (p.skin && !this.skinImages[p.skin]) {
                        this.loadSkin(p.skin);
                    }
                }
                break;

            case 'dead':
                this.running = false;
                if (this.animFrame) {
                    cancelAnimationFrame(this.animFrame);
                }
                UI.showDeath(msg.killer, msg.score);
                break;

            case 'error':
                console.error('Server error:', msg.message);
                break;
        }
    },

    loadSkin(url) {
        const img = new Image();
        img.crossOrigin = 'anonymous';
        img.src = url;
        img.onload = () => {
            this.skinImages[url] = img;
        };
        // Mark as loading to prevent re-requests
        this.skinImages[url] = null;
    },

    updateMouseWorld() {
        const hw = this.canvas.width / 2;
        const hh = this.canvas.height / 2;
        const scale = this.getViewScale();

        this.mouse.worldX = this.camera.x + (this.mouse.x - hw) / scale;
        this.mouse.worldY = this.camera.y + (this.mouse.y - hh) / scale;
    },

    getViewScale() {
        const baseScale = Math.min(this.canvas.width, this.canvas.height) / 800;
        return baseScale / Math.max(0.5, this.camera.scale * 0.5);
    },

    sendMove() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({
                type: 'move',
                x: this.mouse.worldX,
                y: this.mouse.worldY,
            }));
        }
    },

    sendSplit() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({ type: 'split' }));
        }
    },

    sendEject() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({ type: 'eject' }));
        }
    },

    gameLoop() {
        if (!this.running) return;

        this.update();
        this.render();
        this.animFrame = requestAnimationFrame(() => this.gameLoop());
    },

    update() {
        // Smooth camera
        const lerp = 0.08;
        this.camera.x += (this.camera.targetX - this.camera.x) * lerp;
        this.camera.y += (this.camera.targetY - this.camera.y) * lerp;
        this.camera.scale += (this.camera.targetScale - this.camera.scale) * lerp;

        // Interpolation factor
        const elapsed = performance.now() - this.lastStateTime;
        this.interpFactor = Math.min(elapsed / 33.3, 1.0); // 30 TPS
    },

    render() {
        const ctx = this.ctx;
        const w = this.canvas.width;
        const h = this.canvas.height;
        const scale = this.getViewScale();

        // Clear
        ctx.fillStyle = '#0a0a14';
        ctx.fillRect(0, 0, w, h);

        ctx.save();

        // Camera transform
        ctx.translate(w / 2, h / 2);
        ctx.scale(scale, scale);
        ctx.translate(-this.camera.x, -this.camera.y);

        // Draw grid
        this.drawGrid(ctx, scale);

        // Draw world border
        this.drawBorder(ctx);

        // Draw food
        this.drawFood(ctx);

        // Draw viruses
        this.drawViruses(ctx);

        // Draw players (sorted by mass, smallest first)
        const sortedPlayers = [...this.players].sort((a, b) => {
            const massA = a.cells.reduce((s, c) => s + c.radius * c.radius, 0);
            const massB = b.cells.reduce((s, c) => s + c.radius * c.radius, 0);
            return massA - massB;
        });

        for (const player of sortedPlayers) {
            this.drawPlayer(ctx, player);
        }

        ctx.restore();

        // Draw minimap
        this.drawMinimap();
    },

    drawGrid(ctx, scale) {
        const gridSize = 50;
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.04)';
        ctx.lineWidth = 1 / scale;

        const startX = Math.floor((this.camera.x - this.canvas.width / scale / 2) / gridSize) * gridSize;
        const endX = startX + this.canvas.width / scale + gridSize * 2;
        const startY = Math.floor((this.camera.y - this.canvas.height / scale / 2) / gridSize) * gridSize;
        const endY = startY + this.canvas.height / scale + gridSize * 2;

        ctx.beginPath();
        for (let x = startX; x <= endX; x += gridSize) {
            if (x >= 0 && x <= this.worldSize) {
                ctx.moveTo(x, Math.max(0, startY));
                ctx.lineTo(x, Math.min(this.worldSize, endY));
            }
        }
        for (let y = startY; y <= endY; y += gridSize) {
            if (y >= 0 && y <= this.worldSize) {
                ctx.moveTo(Math.max(0, startX), y);
                ctx.lineTo(Math.min(this.worldSize, endX), y);
            }
        }
        ctx.stroke();
    },

    drawBorder(ctx) {
        ctx.strokeStyle = 'rgba(255, 50, 50, 0.4)';
        ctx.lineWidth = 4;
        ctx.strokeRect(0, 0, this.worldSize, this.worldSize);
    },

    drawFood(ctx) {
        for (const f of this.food) {
            ctx.fillStyle = f.color;
            ctx.beginPath();
            ctx.arc(f.x, f.y, 5, 0, Math.PI * 2);
            ctx.fill();
        }
    },

    drawViruses(ctx) {
        for (const v of this.viruses) {
            // Spiked green circle
            const spikes = 20;
            const outerR = v.radius;
            const innerR = v.radius * 0.85;

            ctx.fillStyle = 'rgba(51, 204, 51, 0.3)';
            ctx.strokeStyle = '#33cc33';
            ctx.lineWidth = 2;

            ctx.beginPath();
            for (let i = 0; i < spikes * 2; i++) {
                const angle = (i / (spikes * 2)) * Math.PI * 2;
                const r = i % 2 === 0 ? outerR : innerR;
                const px = v.x + Math.cos(angle) * r;
                const py = v.y + Math.sin(angle) * r;
                if (i === 0) ctx.moveTo(px, py);
                else ctx.lineTo(px, py);
            }
            ctx.closePath();
            ctx.fill();
            ctx.stroke();
        }
    },

    drawPlayer(ctx, player) {
        const isMe = player.id === this.playerId;
        const skinImg = player.skin ? this.skinImages[player.skin] : null;

        for (const cell of player.cells) {
            const x = cell.x;
            const y = cell.y;
            const r = cell.radius;

            // Outer glow for own cells
            if (isMe) {
                ctx.save();
                ctx.shadowColor = 'rgba(46, 204, 64, 0.3)';
                ctx.shadowBlur = 20;
            }

            // Cell body
            ctx.beginPath();
            ctx.arc(x, y, r, 0, Math.PI * 2);

            if (skinImg) {
                // Draw skin image clipped to circle
                ctx.save();
                ctx.clip();
                ctx.drawImage(skinImg, x - r, y - r, r * 2, r * 2);
                ctx.restore();

                // Border
                ctx.strokeStyle = 'rgba(255,255,255,0.3)';
                ctx.lineWidth = 3;
                ctx.beginPath();
                ctx.arc(x, y, r, 0, Math.PI * 2);
                ctx.stroke();
            } else {
                // Gradient fill
                const grad = ctx.createRadialGradient(x - r * 0.3, y - r * 0.3, 0, x, y, r);
                const baseColor = this.getPlayerColor(player);
                grad.addColorStop(0, this.lightenColor(baseColor, 30));
                grad.addColorStop(1, baseColor);
                ctx.fillStyle = grad;
                ctx.fill();

                // Border
                ctx.strokeStyle = this.darkenColor(baseColor, 30);
                ctx.lineWidth = Math.max(2, r * 0.06);
                ctx.stroke();
            }

            if (isMe) {
                ctx.restore();
            }
        }

        // Draw name (on the largest cell)
        if (player.cells.length > 0) {
            const largest = player.cells.reduce((a, b) => a.radius > b.radius ? a : b);
            const fontSize = Math.max(12, largest.radius * 0.4);

            ctx.font = `bold ${fontSize}px 'Segoe UI', sans-serif`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';

            // Text shadow
            ctx.fillStyle = 'rgba(0,0,0,0.5)';
            ctx.fillText(player.name, largest.x + 2, largest.y + 2);

            // Text
            ctx.fillStyle = '#fff';
            ctx.fillText(player.name, largest.x, largest.y);

            // Mass text (smaller, below name)
            if (isMe && largest.radius > 30) {
                const mass = Math.floor((largest.radius / 4) ** 2);
                const smallFont = fontSize * 0.6;
                ctx.font = `${smallFont}px 'Segoe UI', sans-serif`;
                ctx.fillStyle = 'rgba(255,255,255,0.6)';
                ctx.fillText(mass, largest.x, largest.y + fontSize * 0.7);
            }
        }
    },

    getPlayerColor(player) {
        // Generate consistent color from player id
        const colors = [
            '#FF4136', '#FF6B35', '#FFDC00', '#2ECC40', '#0074D9',
            '#7FDBFF', '#B10DC9', '#F012BE', '#FF69B4', '#01FF70',
            '#3D9970', '#39CCCC', '#E65100', '#00BCD4', '#8BC34A',
        ];
        return colors[player.id % colors.length];
    },

    lightenColor(hex, percent) {
        const num = parseInt(hex.slice(1), 16);
        const r = Math.min(255, (num >> 16) + percent);
        const g = Math.min(255, ((num >> 8) & 0xff) + percent);
        const b = Math.min(255, (num & 0xff) + percent);
        return `rgb(${r},${g},${b})`;
    },

    darkenColor(hex, percent) {
        const num = parseInt(hex.slice(1), 16);
        const r = Math.max(0, (num >> 16) - percent);
        const g = Math.max(0, ((num >> 8) & 0xff) - percent);
        const b = Math.max(0, (num & 0xff) - percent);
        return `rgb(${r},${g},${b})`;
    },

    drawMinimap() {
        const miniCanvas = document.getElementById('minimap');
        const mctx = miniCanvas.getContext('2d');
        const mw = miniCanvas.width;
        const mh = miniCanvas.height;
        const scale = mw / this.worldSize;

        mctx.clearRect(0, 0, mw, mh);

        // Background
        mctx.fillStyle = 'rgba(10, 10, 20, 0.8)';
        mctx.fillRect(0, 0, mw, mh);

        // Border
        mctx.strokeStyle = 'rgba(255,255,255,0.2)';
        mctx.lineWidth = 1;
        mctx.strokeRect(0, 0, mw, mh);

        // Draw players as dots
        for (const p of this.players) {
            if (p.cells.length === 0) continue;
            const cx = p.cells.reduce((s, c) => s + c.x, 0) / p.cells.length;
            const cy = p.cells.reduce((s, c) => s + c.y, 0) / p.cells.length;
            const maxR = Math.max(...p.cells.map(c => c.radius));

            const isMe = p.id === this.playerId;
            mctx.fillStyle = isMe ? '#2ECC40' : '#FF4136';
            mctx.beginPath();
            mctx.arc(cx * scale, cy * scale, Math.max(2, maxR * scale), 0, Math.PI * 2);
            mctx.fill();
        }

        // Viewport rectangle
        const viewScale = this.getViewScale();
        const vw = this.canvas.width / viewScale;
        const vh = this.canvas.height / viewScale;
        mctx.strokeStyle = 'rgba(255, 255, 255, 0.5)';
        mctx.lineWidth = 1;
        mctx.strokeRect(
            (this.camera.x - vw / 2) * scale,
            (this.camera.y - vh / 2) * scale,
            vw * scale,
            vh * scale
        );
    },
};

// Initialize game canvas on load
document.addEventListener('DOMContentLoaded', () => Game.init());
