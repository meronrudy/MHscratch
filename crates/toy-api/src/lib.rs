//! Toy API: HTTP server using axum

use axum::{
    routing::post,
    Router,
    Json,
};
use toy_core::{Command, CommandResult, execute};

async fn handle_command(Json(cmd): Json<Command>) -> Json<CommandResult> {
    let res = execute(cmd);
    Json(res)
}

pub async fn serve(addr: String) -> anyhow::Result<()> {
    let app = Router::new().route("/v1/command", post(handle_command));

    println!("Toy API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}