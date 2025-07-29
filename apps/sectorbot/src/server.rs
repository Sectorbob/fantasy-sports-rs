use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};

use axum::{
    Extension, Json, Router,
    response::{IntoResponse, Response},
    routing::get,
};
use reqwest::StatusCode;

use crate::{bot::AppContext, store::Feed};

pub async fn setup_and_listen(
    port: u16,
    app_context: Arc<AppContext>,
) -> Result<(), Box<dyn Error>> {
    // Create the application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/feeds", get(fetch_feeds))
        .layer(Extension(app_context));
    // Define the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Server running at http://{}", addr);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check(
    Extension(state): Extension<Arc<AppContext>>,
) -> Result<Json<Health>, AppError> {
    Ok(Json(Health {
        status: "ok".to_string(),
        message: "Server is running".to_string(),
        yt_credits_used: state.yt_client.credits_consumed(),
    }))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Health {
    pub status: String,
    pub message: String,
    pub yt_credits_used: usize,
}

async fn fetch_feeds(
    // Path(date): Path<NaiveDate>,
    // Query(slate_filters): Query<SlateFilters>,
    Extension(state): Extension<Arc<AppContext>>,
) -> Result<Json<Vec<Feed>>, AppError> {
    match state.store.list_feeds() {
        Ok(feeds) => Ok(Json(feeds)),
        Err(err) => Err(AppError::CustomError(format!(
            "failed to fetch_slates: {err}",
        ))),
    }
}

#[derive(Debug)]
pub(crate) enum AppError {
    BadRequest(String),
    NotFound(String),
    CustomError(String),
}

// Implement `IntoResponse` for `AppError` to customize the HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
            AppError::CustomError(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
            }
        }
    }
}
