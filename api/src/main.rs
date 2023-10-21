use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use dirs::home_dir;
use store::{self, types::EventWithTaskName};

#[derive(Clone)]
struct AppState {
    store: store::Store,
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = router().await;
    // run it with hyper on localhost:3000
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
        .route("/", get(|| async { "Hello, World!" }))
        .route("/events", get(get_events))
        .with_state(state)
}

async fn get_events(state: State<AppState>) -> Json<Vec<EventWithTaskName>> {
    let events = state.store.clone().get_events().await.unwrap();
    Json(events)
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
