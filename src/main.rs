use scratch_server::{Cors, HttpServer};

mod api;

fn main() {
    HttpServer::build()
        .with_cors_policy(
            Cors::new()
                .with_origins("*")
                .with_methods("GET, POST, PUT, DELETE")
                .with_headers("Content-Type, Authorization")
                .with_credentials("true"),
        )
        .add_routes(api::create_routes())
        .run()
        .expect("Starting server failed");
}
