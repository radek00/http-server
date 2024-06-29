use scratch_server::{Cors, HttpServer};
use serde::{Deserialize, Serialize};

mod api;
#[derive(Debug, Serialize, Deserialize)]
struct Example {
    message: String,
}

fn main() {
    let mut http_server = HttpServer::build();
    api::create_routes(&mut http_server.router);

    http_server
        .with_cors_policy(
            Cors::new()
                .with_origins("*")
                .with_methods("GET, POST, PUT, DELETE")
                .with_headers("Content-Type, Authorization")
                .with_credentials("true"),
        )
        .run()
        .expect("Starting server failed");
}
