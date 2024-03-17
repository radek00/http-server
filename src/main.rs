use http_server::HttpServer;

fn main() {
    let server = HttpServer {
        port: 7878,
        threads: 4,
    };
    server.run().expect("Starting server failed");
}
