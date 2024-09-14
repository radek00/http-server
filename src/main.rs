use api::build_server;

mod api;

fn main() {
    let server = build_server();
    server
        .0
        .add_routes(api::create_routes(server.1, server.2))
        .run()
        .expect("Starting server failed");
}
