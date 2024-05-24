use http_server::HttpServer;
use serde::{Deserialize, Serialize};

mod api;
#[derive(Debug, Serialize, Deserialize)]
struct Example {
    message: String,
}

fn main() {
    let mut http_server = HttpServer::build().with_logger();
    api::create_routes(&mut http_server.router);

    http_server.run().expect("Starting server failed");
}
