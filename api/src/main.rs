use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use dirs::home_dir;
use store::{
    self,
    types::{Event, EventWithTaskName, Task},
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
        .route("/add-event", get(add_event))
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

async fn add_event(state: State<AppState>) -> Result<Json<Event>, (StatusCode, String)> {
    let res = state
        .store
        .add_task_event(
            "silks".to_string(),
            "testing from the api".to_string(),
            "2023-10-22 18:03:06.722619025 -07:00".to_string(),
            4,
        )
        .await
        .map_err(internal_error)?;
    print!("{:?}", res);
    Ok(Json(res))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
