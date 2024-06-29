use api::build_server;

mod api;

fn main() {
    build_server()
        .add_routes(api::create_routes())
        .run()
        .expect("Starting server failed");
}
