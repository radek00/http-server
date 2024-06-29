use api::build_server;
use scratch_server::Cors;

mod api;

fn main() {
    build_server()
        .with_cors_policy(
            Cors::new()
                .with_origins("https://www.example.com, https://www.example2.com")
                .with_methods("GET, POST, PUT, DELETE")
                .with_headers("Content-Type, Authorization")
                .with_credentials("true"),
        )
        .add_routes(api::create_routes())
        .run()
        .expect("Starting server failed");
}
