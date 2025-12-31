use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

const WINNING_COMBOS: [[usize; 3]; 8] = [
    [0, 1, 2], [3, 4, 5], [6, 7, 8], // rows
    [0, 3, 6], [1, 4, 7], [2, 5, 8], // cols
    [0, 4, 8], [2, 4, 6],            // diagonals
];

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Game {
    pub id: i32,
    pub board: sqlx::types::Json<Vec<String>>,
    pub current_player: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct GameResponse {
    pub id: i32,
    pub board: Vec<Option<String>>,
    pub current_player: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_display: Option<String>,
}

impl Game {
    pub fn to_response(&self, include_display: bool) -> GameResponse {
        let board: Vec<Option<String>> = self.board.0.iter()
            .map(|s| if s.is_empty() { None } else { Some(s.clone()) })
            .collect();

        GameResponse {
            id: self.id,
            board,
            current_player: self.current_player.clone(),
            status: self.status.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            board_display: if include_display { Some(self.get_board_display()) } else { None },
        }
    }

    pub async fn create(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let game = sqlx::query_as::<_, Game>(
            r#"
            INSERT INTO games (board, current_player, status)
            VALUES ($1, 'X', 'in_progress')
            RETURNING *
            "#,
        )
        .bind(sqlx::types::Json(vec!["".to_string(); 9]))
        .fetch_one(pool)
        .await?;

        Ok(game)
    }

    pub async fn find_by_id(pool: &PgPool, id: i32) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Game>("SELECT * FROM games ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }

    pub async fn delete(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM games WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn make_move(&mut self, pool: &PgPool, position: usize) -> Result<(), String> {
        // Validate game is in progress
        if self.status != "in_progress" {
            return Err("Game is already finished".to_string());
        }

        // Validate position
        if position > 8 {
            return Err("Invalid position. Must be 0-8".to_string());
        }

        // Check if position is occupied
        if !self.board.0[position].is_empty() {
            return Err("Position already occupied".to_string());
        }

        // Make the move
        self.board.0[position] = self.current_player.clone();

        // Check for winner
        if let Some(winner) = self.check_winner() {
            self.status = format!("{}_wins", winner.to_lowercase());
        } else if self.is_draw() {
            self.status = "draw".to_string();
        } else {
            // Switch player
            self.current_player = if self.current_player == "X" { "O".to_string() } else { "X".to_string() };
        }

        // Save to database
        sqlx::query(
            "UPDATE games SET board = $1, current_player = $2, status = $3, updated_at = NOW() WHERE id = $4"
        )
        .bind(&self.board)
        .bind(&self.current_player)
        .bind(&self.status)
        .bind(self.id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn check_winner(&self) -> Option<String> {
        for combo in WINNING_COMBOS {
            let a = &self.board.0[combo[0]];
            let b = &self.board.0[combo[1]];
            let c = &self.board.0[combo[2]];

            if !a.is_empty() && a == b && b == c {
                return Some(a.clone());
            }
        }
        None
    }

    pub fn is_draw(&self) -> bool {
        self.check_winner().is_none() && self.board.0.iter().all(|cell| !cell.is_empty())
    }

    pub fn get_board_display(&self) -> String {
        let b = &self.board.0;
        let cell = |i: usize| if b[i].is_empty() { ".".to_string() } else { b[i].clone() };

        format!(
            " {} | {} | {}\n-----------\n {} | {} | {}\n-----------\n {} | {} | {}",
            cell(0), cell(1), cell(2),
            cell(3), cell(4), cell(5),
            cell(6), cell(7), cell(8)
        )
    }
}
