CREATE TABLE games (
    id SERIAL PRIMARY KEY,
    board JSONB NOT NULL DEFAULT '["", "", "", "", "", "", "", "", ""]',
    current_player VARCHAR(1) NOT NULL DEFAULT 'X',
    status VARCHAR(20) NOT NULL DEFAULT 'in_progress',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_games_status ON games(status);
