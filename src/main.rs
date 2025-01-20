mod application;
mod domain;
mod infrastructure;
mod presentation;

use crate::infrastructure::server::create_app;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = create_app().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tracing::info!("Server running on http://0.0.0.0:8000");
    
    axum::serve(listener, app).await.unwrap();
}
