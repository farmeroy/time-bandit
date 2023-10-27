use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use dirs::home_dir;
use http::{header, Method, Request, Response};
use hyper::Body;
use std::{convert::Infallible, net::SocketAddr};
use store::{
    self,
    types::{Event, EventWithTaskName, NewEvent, NewEventWithTaskName, Task},
};
use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{event, info, instrument, Level};

#[derive(Clone)]
struct AppState {
    store: store::Store,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    // build our application with a single route
    let app = router().await;
    let addr = SocketAddr::from(([0, 0, 0, 0], 7878));
    // run it with hyper on localhost:7878
    info!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn router() -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);
    let home_dir = home_dir().unwrap();
    let state = AppState {
        store: store::Store::new(
            format!("{}/.time_bandit.db3", home_dir.to_string_lossy()).as_str(),
        )
        .await
        .unwrap(),
    };
    Router::new()
        .layer(TraceLayer::new_for_http())
        .route("/", get(|| async { "Time Bandit" }))
        .route("/events", get(get_events))
        .route("/tasks", get(get_tasks))
        .route("/add-event", post(add_event))
        .with_state(state)
        .layer(cors)
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

async fn add_event(
    state: State<AppState>,
    event: Json<NewEventWithTaskName>,
) -> Result<Json<Event>, (StatusCode, String)> {
    let res = state
        .store
        .add_task_event(
            event.task_name.to_string(),
            event.event.clone().notes.unwrap_or_default(),
            event.event.time_stamp.to_string(),
            event.event.duration as u64,
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
