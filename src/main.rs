use http_server::HttpServer;
use serde::{Deserialize, Serialize};

mod api;
#[derive(Debug, Serialize, Deserialize)]
struct Example {
    message: String,
}

fn main() {
    let router = api::create_routes();

    HttpServer::build()
        .run(router)
        .expect("Starting server failed");
}
