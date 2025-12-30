use std::net::TcpListener;
use toy_api::serve;

#[tokio::test]
async fn test_api_get_instances() {
    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener); // Free the port

    let addr_str = format!("{}:{}", addr.ip(), addr.port());

    // Spawn the server in background
    tokio::spawn(async move {
        serve(addr_str).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make HTTP request
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://127.0.0.1:{}/v1/instances", addr.port()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["ok"].as_bool().unwrap());
    assert!(body["data"]["instances"].is_array());
    assert!(body["data"]["instances"].as_array().unwrap().contains(&serde_json::json!("triangle_attractor_v0")));
}

#[tokio::test]
async fn test_api_get_status() {
    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let addr_str = format!("{}:{}", addr.ip(), addr.port());

    // Spawn the server
    tokio::spawn(async move {
        serve(addr_str).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://127.0.0.1:{}/v1/status", addr.port()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["ok"].as_bool().unwrap());
    assert_eq!(body["message"], "status");
    assert!(body["data"]["version"].is_string());
}

#[tokio::test]
async fn test_api_post_command() {
    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let addr_str = format!("{}:{}", addr.ip(), addr.port());

    // Spawn the server
    tokio::spawn(async move {
        serve(addr_str).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let command = serde_json::json!({
        "ListInstances": {}
    });

    let response = client
        .post(&format!("http://127.0.0.1:{}/v1/command", addr.port()))
        .json(&command)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["ok"].as_bool().unwrap());
    assert!(body["data"]["instances"].is_array());
}

#[tokio::test]
async fn test_api_unknown_endpoint() {
    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let addr_str = format!("{}:{}", addr.ip(), addr.port());

    // Spawn the server
    tokio::spawn(async move {
        serve(addr_str).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://127.0.0.1:{}/v1/unknown", addr.port()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}