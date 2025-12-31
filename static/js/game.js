// TicTacToe JavaScript utilities
const TicTacToe = {
    async createGame() {
        const response = await fetch('/api/games', { method: 'POST' });
        return response.json();
    },

    async loadGame(gameId) {
        const response = await fetch('/api/games/' + gameId);
        return response.json();
    },

    async makeMove(gameId, position) {
        const response = await fetch('/api/games/' + gameId + '/move', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ position })
        });
        return { ok: response.ok, data: await response.json() };
    },

    async deleteGame(gameId) {
        const response = await fetch('/api/games/' + gameId, { method: 'DELETE' });
        return response.ok;
    }
};
