// src/server.rs
// HTTP server module for Cyre

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use serde_json::json;

/// Simple HTTP server for Cyre
pub struct CyreServer {
    pub addr: SocketAddr,
}

impl CyreServer {
    /// Create a new Cyre server
    pub fn new(port: u16) -> Self {
        Self {
            addr: SocketAddr::from(([0, 0, 0, 0], port)),
        }
    }

    /// Start the server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let make_svc = make_service_fn(|_conn| async {
            Ok::<_, Infallible>(service_fn(handle_request))
        });

        let server = Server::bind(&self.addr).serve(make_svc);

        println!("🌐 Cyre server running on http://{}", self.addr);

        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }

        Ok(())
    }
}

/// Handle HTTP requests
async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response_body = json!({
        "message": "Cyre Rust Server",
        "status": "running",
        "timestamp": crate::utils::current_timestamp()
    });

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(response_body.to_string()))
        .unwrap();

    Ok(response)
}

/// Run the server with default settings
pub async fn run_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = CyreServer::new(3000);
    server.start().await
}