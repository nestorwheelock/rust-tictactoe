use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use sqlx::PgPool;

use crate::models::Game;

#[derive(Template)]
#[template(path = "game_list.html")]
pub struct GameListTemplate {
    pub games: Vec<Game>,
}

#[derive(Template)]
#[template(path = "game_detail.html")]
pub struct GameDetailTemplate {
    pub game: Game,
}

#[derive(Template)]
#[template(path = "not_found.html")]
pub struct NotFoundTemplate;

// GET / - Game list page
pub async fn game_list(State(pool): State<PgPool>) -> impl IntoResponse {
    let games = Game::list_all(&pool).await.unwrap_or_default();
    GameListTemplate { games }
}

// GET /game/:id - Game detail page
pub async fn game_detail(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match Game::find_by_id(&pool, id).await {
        Ok(Some(game)) => GameDetailTemplate { game }.into_response(),
        _ => NotFoundTemplate.into_response(),
    }
}
