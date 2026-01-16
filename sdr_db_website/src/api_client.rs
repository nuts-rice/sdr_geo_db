use gloo_net::http::{Request, Response};
use gloo_net::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use sdr_db::{api_types::{LogResponse, LogsResponse}, LogFormData};

pub async fn get_logs_async(base_url: &str) -> Result<Vec<LogResponse>, Error> {
    let response = Request::get(base_url).send().await?;
    let body = response.text().await?;
    let parsed: LogsResponse = serde_json::from_str(&body).map_err(|e| {
        gloo_net::Error::SerdeError(e)
    })?;
    Ok(parsed.logs)
}

pub async fn create_log_async(
    base_url: &str,
    form_data: LogFormData,
) -> Result<LogResponse, Error> {
    let request_body = serde_json::to_string(&form_data).unwrap();
    let response = Request::post(base_url)
        .header("Content-Type", "application/json")
        .body(request_body)?
        .send()
        .await
        .unwrap();
    let body = response.text().await?;
    let body: LogResponse = serde_json::from_str(&body).unwrap();
    Ok(body)
}
