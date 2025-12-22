use axum::{routing::get_service, Router};

use anyhow::Result;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

pub async fn start(root: String) -> Result<()> {
    println!("Serving: {}", root);

    let app = Router::new()
        .nest_service("/", get_service(ServeDir::new(root)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));

    println!("Server running on http://localhost:9000");

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app
    ).await.unwrap();

    Ok(())
}
