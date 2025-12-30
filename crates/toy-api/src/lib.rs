//! Toy API: HTTP server using axum

use axum::{
    routing::{get, post},
    Router,
    Json,
};
use toy_core::{Command, CommandResult, execute};

async fn handle_command(Json(cmd): Json<Command>) -> Json<CommandResult> {
    let res = execute(cmd);
    Json(res)
}

async fn get_instances() -> Json<CommandResult> {
    let res = execute(Command::ListInstances);
    Json(res)
}

async fn get_status() -> Json<CommandResult> {
    let res = execute(Command::GetStatus);
    Json(res)
}

pub async fn serve(addr: String) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/v1/command", post(handle_command))
        .route("/v1/instances", get(get_instances))
        .route("/v1/status", get(get_status));

    println!("Toy API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}