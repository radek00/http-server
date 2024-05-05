use http_server::HttpServer;
use serde::{Deserialize, Serialize};

mod api;
#[derive(Debug, Serialize, Deserialize)]
struct Example {
    message: String,
}

fn main() {
    let server = HttpServer {
        port: 7878,
        threads: 4,
    };

    let router  = api::create_routes();

    server.run(router).expect("Starting server failed");
}