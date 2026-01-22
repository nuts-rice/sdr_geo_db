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
use diesel::r2d2::{self, ConnectionManager, Pool};
use sdr_db::api_types::{ErrorResponse, LogResponse, LogsResponse};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use sdr_db::{Log, LogFormData, create_log, get_logs, spectrum_types::generate_mock_spectrum};

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
}

impl Config {
    fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

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
        })
    }
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
    State(pool): State<DbPool>,
    Json(form_data): Json<LogFormData>,
) -> Result<(StatusCode, Json<LogResponse>), AppError> {
    form_data.validate().map_err(AppError::Validation)?;

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
    State(pool): State<DbPool>,
    Query(params): Query<LogQueryParams>,
) -> Result<Json<LogsResponse>, AppError> {
    let limit = params.limit.min(1000);

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

async fn health_handler(State(pool): State<DbPool>) -> Result<Json<HealthResponse>, AppError> {
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

async fn spectrum_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_spectrum_ws)
}

async fn handle_spectrum_ws(mut socket: WebSocket) {
    let mut interval = tokio::time::interval(Duration::from_millis(66));
    let start = tokio::time::Instant::now();
    loop {
        interval.tick().await;
        let elapsed = start.elapsed().as_secs_f64();
        let frame = generate_mock_spectrum(elapsed);
        let msg = serde_json::to_string(&frame).unwrap();
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
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

    // Create db pool
    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let pool = Pool::builder().max_size(10).build(manager)?;

    info!("Database pool created");

    let cors = CorsLayer::new()
        .allow_origin(
            config
                .allowed_origins
                .iter()
                .map(|s| s.parse().unwrap())
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
        .with_state(pool);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
