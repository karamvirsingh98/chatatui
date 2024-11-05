use axum::{
    extract::{
        ws::{Message, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::{any, get},
    Json, Router,
};
use tokio::sync::broadcast;

use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use futures::{stream::StreamExt, SinkExt};

#[derive(Clone)]
pub struct AppState {
    pub messages: Vec<chatatui_types::Message>,
    pub sender: broadcast::Sender<String>,
    pub receiver: Arc<broadcast::Receiver<String>>,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (sender, receiver) = broadcast::channel::<String>(1000);

    // initialize app state
    let state = AppState {
        messages: Vec::new(),
        sender,
        receiver: Arc::new(receiver),
    };

    // build our application with a route
    let app = Router::new()
        .route("/", get(get_messages))
        .route("/ws", any(ws_handler))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn get_messages(State(state): State<AppState>) -> Json<Vec<chatatui_types::Message>> {
    state.messages.clone().into()
}

async fn ws_handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> impl IntoResponse {
    let mut rx = app.receiver.resubscribe();
    ws.on_upgrade(move |socket| async {
        let (mut sender, mut receiver) = socket.split();
        tokio::spawn(async move {
            while let Some(Ok(Message::Text(msg))) = receiver.next().await {
                let res = app.sender.send(msg);
                if let Err(e) = res {
                    eprintln!("{e:#?}")
                }
            }
        });
        tokio::spawn(async move {
            loop {
                if let Ok(msg) = rx.recv().await {
                    let res = sender.send(Message::Text(msg)).await;
                    if let Err(e) = res {
                        eprintln!("{e:#?}");
                        break;
                    }
                };
            }
        });
    })
}
