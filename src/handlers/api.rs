use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::Game;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Deserialize)]
pub struct MoveRequest {
    pub position: usize,
}

// POST /api/games - Create new game
pub async fn create_game(State(pool): State<PgPool>) -> impl IntoResponse {
    match Game::create(&pool).await {
        Ok(game) => (StatusCode::CREATED, Json(game.to_response(false))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        ).into_response(),
    }
}

// GET /api/games - List all games
pub async fn list_games(State(pool): State<PgPool>) -> impl IntoResponse {
    match Game::list_all(&pool).await {
        Ok(games) => {
            let responses: Vec<_> = games.iter().map(|g| g.to_response(false)).collect();
            Json(responses).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        ).into_response(),
    }
}

// GET /api/games/:id - Get game details
pub async fn get_game(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match Game::find_by_id(&pool, id).await {
        Ok(Some(game)) => Json(game.to_response(true)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "Game not found".to_string() }),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        ).into_response(),
    }
}

// DELETE /api/games/:id - Delete game
pub async fn delete_game(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match Game::delete(&pool, id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "Game not found".to_string() }),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        ).into_response(),
    }
}

// POST /api/games/:id/move - Make a move
pub async fn make_move(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(payload): Json<MoveRequest>,
) -> impl IntoResponse {
    let game = match Game::find_by_id(&pool, id).await {
        Ok(Some(g)) => g,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse { error: "Game not found".to_string() }),
            ).into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e.to_string() }),
            ).into_response()
        }
    };

    let mut game = game;
    match game.make_move(&pool, payload.position).await {
        Ok(()) => Json(game.to_response(true)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        ).into_response(),
    }
}
