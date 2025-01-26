mod application;
mod domain;
mod infrastructure;
mod presentation;

use crate::infrastructure::server::create_app;
use crate::infrastructure::constants::InfrastructureConstants;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = create_app().await;
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| InfrastructureConstants::DEFAULT_PORT.to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server running on http://{}", addr);
    
    axum::serve(listener, app).await.unwrap();
}
