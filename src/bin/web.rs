use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Database demo" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    println!("app running on port 3030");
    axum::serve(listener, app).await.unwrap();
}
