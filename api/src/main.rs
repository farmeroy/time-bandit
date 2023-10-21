use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use dirs::home_dir;
use store::{
    self,
    types::{EventWithTaskName, Task},
};

#[derive(Clone)]
struct AppState {
    store: store::Store,
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = router().await;
    // run it with hyper on localhost:7878
    axum::Server::bind(&"0.0.0.0:7878".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn router() -> Router {
    let home_dir = home_dir().unwrap();
    let state = AppState {
        store: store::Store::new(
            format!("{}/.time_bandit.db3", home_dir.to_string_lossy()).as_str(),
        )
        .await
        .unwrap(),
    };
    Router::new()
        .route("/", get(|| async { "Time Bandit" }))
        .route("/events", get(get_events))
        .route("/tasks", get(get_tasks))
        .with_state(state)
}

async fn get_events(
    state: State<AppState>,
) -> Result<Json<Vec<EventWithTaskName>>, (StatusCode, String)> {
    let res = state.store.get_events().await.map_err(internal_error)?;
    Ok(Json(res))
}

async fn get_tasks(state: State<AppState>) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    let res = state.store.get_tasks().await.map_err(internal_error)?;
    Ok(Json(res))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
