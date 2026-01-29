use axum::{
    Json, Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use sdr_db::api_types::{ErrorResponse, LogResponse, LogsResponse};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

use sdr_db::source::sdr::{SdrDevice, SdrSource};
use sdr_db::spectrum_types::SpectrumFrame;
use sdr_db::{Log, LogFormData, create_log, get_logs};

/*
*#post log : curl -X POST http://localhost:3000/logs -H "Content-Type: application/json" -d '{"frequency": 14.070, "grid_square": "FN31", "callsign": "K1ABC", "mode": "USB", "comment": "Test log entry", "recording_duration": 120.5}'
*#get logs : curl -X GET 'http://localhost:3000/logs?limit=10&mode=FT8&grid=FN31'
*#health check : curl -X GET http://localhost:3000/health
*/
type DbPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
struct Config {
    database_url: String,
    port: u16,
    allowed_origins: Vec<String>,
    sdr_device: SdrDevice,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let sdr_device = std::env::var("SDR_MODE")
            .ok()
            .and_then(|s| SdrDevice::from_str(&s).ok())
            .unwrap_or_default();

        Ok(Config {
            database_url: std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set")?,
            port: std::env::var("API_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| "Invalid API_PORT")?,
            allowed_origins: std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:8080".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            sdr_device,
        })
    }
}

/// Shared application state
#[derive(Clone)]
struct AppState {
    db_pool: DbPool,
    spectrum_tx: broadcast::Sender<SpectrumFrame>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    database: String,
}

#[derive(Deserialize)]
struct LogQueryParams {
    #[serde(default = "default_limit")]
    limit: i64,
    mode: Option<String>,
    grid: Option<String>,
}

fn default_limit() -> i64 {
    100
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Pool error: {0}")]
    Pool(#[from] diesel::r2d2::PoolError),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Database(e) => {
                error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            AppError::Pool(e) => {
                error!("Pool error: {}", e);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Service unavailable".to_string(),
                )
            }
            AppError::Internal(msg) => {
                error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal error".to_string(),
                )
            }
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

async fn create_log_handler(
    State(state): State<AppState>,
    Json(form_data): Json<LogFormData>,
) -> Result<(StatusCode, Json<LogResponse>), AppError> {
    form_data.validate().map_err(AppError::Validation)?;

    let pool = state.db_pool.clone();
    let log = tokio::task::spawn_blocking(move || -> Result<Log, AppError> {
        let mut conn = pool.get()?;
        let log = create_log(
            &mut conn,
            form_data.frequency,
            form_data.grid_square,
            form_data.callsign,
            form_data.mode,
            form_data.comment,
            form_data.recording_duration,
        )?;
        Ok(log)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    info!("Created log entry: id={}", log.id);
    Ok((StatusCode::CREATED, Json(log.into())))
}

async fn list_logs_handler(
    State(state): State<AppState>,
    Query(params): Query<LogQueryParams>,
) -> Result<Json<LogsResponse>, AppError> {
    let limit = params.limit.min(1000);

    let pool = state.db_pool.clone();
    let logs = tokio::task::spawn_blocking(move || -> Result<Vec<Log>, AppError> {
        let mut conn = pool.get()?;
        let mut logs = get_logs(&mut conn, limit)?;

        if let Some(mode_filter) = params.mode {
            logs.retain(|log| log.mode.eq_ignore_ascii_case(&mode_filter));
        }

        if let Some(grid_filter) = params.grid {
            logs.retain(|log| {
                log.grid
                    .as_ref()
                    .map(|g| g.starts_with(&grid_filter))
                    .unwrap_or(false)
            });
        }

        Ok(logs)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    let count = logs.len();
    let log_responses: Vec<LogResponse> = logs.into_iter().map(Into::into).collect();

    Ok(Json(LogsResponse {
        logs: log_responses,
        count,
    }))
}

async fn health_handler(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    let pool = state.db_pool.clone();
    let db_status = tokio::task::spawn_blocking(move || -> Result<String, AppError> {
        let mut conn = pool.get()?;
        diesel::sql_query("SELECT 1").execute(&mut conn)?;
        Ok("connected".to_string())
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status,
    }))
}

async fn spectrum_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    let rx = state.spectrum_tx.subscribe();
    ws.on_upgrade(move |socket| handle_spectrum_ws(socket, rx))
}

async fn handle_spectrum_ws(mut socket: WebSocket, mut rx: broadcast::Receiver<SpectrumFrame>) {
    loop {
        match rx.recv().await {
            Ok(frame) => {
                let msg = serde_json::to_string(&frame).unwrap();
                if socket.send(Message::Text(msg)).await.is_err() {
                    break; // Client disconnected
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                warn!("WebSocket client lagged, skipped {} frames", n);
                // Continue receiving
            }
            Err(broadcast::error::RecvError::Closed) => {
                break; // Channel closed
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("api=debug".parse()?),
        )
        .init();

    let config = Config::from_env()?;
    info!("Starting API server on port {}", config.port);
    info!("Allowed origins: {:?}", config.allowed_origins);
    info!("SDR mode: {:?}", config.sdr_device);

    // Create db pool
    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let db_pool = Pool::builder().max_size(10).build(manager)?;
    info!("Database pool created");

    // Create broadcast channel for spectrum data (buffer 16 frames)
    let (spectrum_tx, _) = broadcast::channel::<SpectrumFrame>(16);

    // Start SDR source - it will push frames to the broadcast channel
    let mut sdr_source = SdrSource::new(config.sdr_device.clone(), None)
        .map_err(|e| format!("Failed to create SDR source: {}", e))?;
    info!("SDR source created");

    // Spawn task to forward frames from SdrSource to broadcast channel
    let tx_clone = spectrum_tx.clone();
    tokio::spawn(async move {
        while let Some(frame) = sdr_source.recv().await {
            // Ignore send errors (no subscribers)
            let _ = tx_clone.send(frame);
        }
        warn!("SDR source stopped");
    });

    let state = AppState {
        db_pool,
        spectrum_tx,
    };

    let cors = CorsLayer::new()
        .allow_origin(
            config
                .allowed_origins
                .iter()
                .map(|s| s.parse::<HeaderValue>().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(
            "https://sdr-geo-db.vercel.app"
                .parse::<HeaderValue>()
                .unwrap(),
        )
        .allow_headers(Any);

    let app = Router::new()
        .route("/logs", post(create_log_handler))
        .route("/logs", get(list_logs_handler))
        .route("/health", get(health_handler))
        .route("/spectrum", get(spectrum_handler))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
