// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Module: main
// Description: Entry point for the JMAP Scheduling Server, initializing the Axum HTTP server and routes.

mod auth;
mod crypto;
mod db;
mod jmap;
mod models;
mod nats;

use axum::{routing::{get, post}, Router, response::Response, middleware::map_response};
use axum::http::HeaderValue;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any, AllowOrigin};
use axum::http::{Method, header::{AUTHORIZATION, CONTENT_TYPE, ACCEPT}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Middleware to inject the "Canary" watermark into every HTTP response
async fn inject_canary_header(mut response: Response) -> Response {
    response.headers_mut().insert("X-Powered-By", HeaderValue::from_static("Fastcomcorp-Casin-Engine"));
    response
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "jmap_scheduler=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Setup DB pool
    // let db_url = env::var("JMAP_DATABASE_URL").expect("JMAP_DATABASE_URL must be set");
    // let pool = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect(&db_url)
    //     .await?;

    // db::migrations::run_migrations(&pool).await?;

    // API Gateway: Secure Cross-Origin Resource Sharing (CORS)
    // Only allow our official frontend domain to make API calls
    let cors = CorsLayer::new()
        .allow_origin("https://app.domain.com".parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT]);

    // Setup routes
    let app = Router::new()
        .route("/.well-known/jmap", get(jmap::session::handle_session))
        .route("/jmap", post(jmap::methods::handle_jmap))
        .layer(cors)
        .layer(map_response(inject_canary_header));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
