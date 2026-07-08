use api::{build_server, ServerConfig};
mod api;

fn main() {
    let ServerConfig {
        server,
        authorize,
        index_path,
    } = build_server();
    server
        .add_routes(api::create_routes(authorize, index_path))
        .run()
        .expect("Starting server failed");
}
