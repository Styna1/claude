// ── UI Controller ──

const UI = {
    menuOverlay: null,
    deathOverlay: null,
    hud: null,
    playerNameInput: null,
    authMessage: null,
    loggedInUser: null,
    sessionToken: null,

    init() {
        this.menuOverlay = document.getElementById('menuOverlay');
        this.deathOverlay = document.getElementById('deathOverlay');
        this.hud = document.getElementById('hud');
        this.playerNameInput = document.getElementById('playerName');
        this.authMessage = document.getElementById('authMessage');

        // Play button
        document.getElementById('playBtn').addEventListener('click', () => this.play());
        document.getElementById('respawnBtn').addEventListener('click', () => this.play());

        // Enter key to play
        this.playerNameInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') this.play();
        });

        // Auth buttons
        document.getElementById('loginBtn').addEventListener('click', () => this.login());
        document.getElementById('registerBtn').addEventListener('click', () => this.register());
        document.getElementById('logoutBtn').addEventListener('click', () => this.logout());

        // Skin upload
        document.getElementById('skinUpload').addEventListener('change', (e) => this.uploadSkin(e));

        // Check if already logged in
        this.checkSession();
    },

    async checkSession() {
        try {
            const res = await fetch('/api/me');
            const data = await res.json();
            if (data.ok) {
                this.setLoggedIn(data.username, data.user_id);
            }
        } catch (e) {
            console.log('Not logged in');
        }
    },

    setLoggedIn(username, userId) {
        this.loggedInUser = { username, userId };
        this.sessionToken = this.getCookie('session');

        document.getElementById('loginForm').style.display = 'none';
        document.getElementById('loggedInInfo').style.display = 'block';
        document.getElementById('loggedInName').textContent = username;
        document.getElementById('authStatus').textContent = `Logged in as ${username}`;
        this.playerNameInput.value = username;

        // Show skin preview
        const preview = document.getElementById('skinPreview');
        const img = new Image();
        img.src = `/api/skin/${userId}?t=${Date.now()}`;
        img.onload = () => {
            preview.innerHTML = '';
            preview.appendChild(img);
            preview.style.display = 'block';
        };
        img.onerror = () => {
            preview.style.display = 'none';
        };
    },

    setLoggedOut() {
        this.loggedInUser = null;
        this.sessionToken = null;
        document.getElementById('loginForm').style.display = 'block';
        document.getElementById('loggedInInfo').style.display = 'none';
        document.getElementById('authStatus').textContent = '';
        document.getElementById('skinPreview').style.display = 'none';
    },

    async login() {
        const username = document.getElementById('authUsername').value.trim();
        const password = document.getElementById('authPassword').value;
        if (!username || !password) {
            this.showAuthMsg('Please fill in all fields', false);
            return;
        }
        try {
            const res = await fetch('/api/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
            });
            const data = await res.json();
            if (data.ok) {
                this.showAuthMsg('Logged in!', true);
                this.setLoggedIn(data.username, data.user_id);
            } else {
                this.showAuthMsg(data.message, false);
            }
        } catch (e) {
            this.showAuthMsg('Connection error', false);
        }
    },

    async register() {
        const username = document.getElementById('authUsername').value.trim();
        const password = document.getElementById('authPassword').value;
        if (!username || !password) {
            this.showAuthMsg('Please fill in all fields', false);
            return;
        }
        try {
            const res = await fetch('/api/register', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
            });
            const data = await res.json();
            if (data.ok) {
                this.showAuthMsg('Account created! You can now login.', true);
            } else {
                this.showAuthMsg(data.message, false);
            }
        } catch (e) {
            this.showAuthMsg('Connection error', false);
        }
    },

    async logout() {
        try {
            await fetch('/api/logout', { method: 'POST' });
        } catch (e) {}
        this.setLoggedOut();
    },

    async uploadSkin(event) {
        const file = event.target.files[0];
        if (!file) return;
        if (file.size > 256 * 1024) {
            alert('File too large! Max 256KB.');
            return;
        }
        const formData = new FormData();
        formData.append('skin', file);
        try {
            const res = await fetch('/api/skin', {
                method: 'POST',
                body: formData,
            });
            if (res.ok) {
                // Refresh preview
                if (this.loggedInUser) {
                    const preview = document.getElementById('skinPreview');
                    const img = new Image();
                    img.src = `/api/skin/${this.loggedInUser.userId}?t=${Date.now()}`;
                    img.onload = () => {
                        preview.innerHTML = '';
                        preview.appendChild(img);
                        preview.style.display = 'block';
                    };
                }
            } else {
                alert('Failed to upload skin');
            }
        } catch (e) {
            alert('Upload error');
        }
    },

    play() {
        const name = this.playerNameInput.value.trim() || 'Unnamed';
        this.menuOverlay.style.display = 'none';
        this.deathOverlay.style.display = 'none';
        this.hud.style.display = 'block';

        // Start game
        Game.connect(name, this.sessionToken);
    },

    showDeath(killer, score) {
        const info = document.getElementById('deathInfo');
        const scoreEl = document.getElementById('deathScore');
        info.textContent = killer ? `Eaten by ${killer}` : 'You were consumed!';
        scoreEl.textContent = `Final Score: ${score}`;
        this.deathOverlay.style.display = 'flex';
        this.hud.style.display = 'none';
    },

    showMenu() {
        this.menuOverlay.style.display = 'flex';
        this.deathOverlay.style.display = 'none';
        this.hud.style.display = 'none';
    },

    updateLeaderboard(entries, myId) {
        const list = document.getElementById('leaderboardList');
        list.innerHTML = '';
        entries.forEach(entry => {
            const li = document.createElement('li');
            li.textContent = `${entry.name} — ${entry.score}`;
            list.appendChild(li);
        });
    },

    updateScore(score) {
        document.getElementById('scoreDisplay').textContent = `Score: ${score}`;
    },

    showAuthMsg(msg, success) {
        this.authMessage.textContent = msg;
        this.authMessage.className = 'auth-message ' + (success ? 'success' : 'error');
    },

    getCookie(name) {
        const match = document.cookie.match(new RegExp('(^| )' + name + '=([^;]+)'));
        return match ? match[2] : null;
    },
};

document.addEventListener('DOMContentLoaded', () => UI.init());
