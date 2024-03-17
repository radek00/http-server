use http_server::{HttpServer, router::Router};

fn main() {
    let server = HttpServer {
        port: 7878,
        threads: 4,
    };
    let mut router = Router::new();
    router.add_route("/","GET", |data| {
        println!("Request to root path with data {}",data);
        Ok(())
    });
    router.add_route("/other", "GET", |data| {
        println!("Request to other path with data {}",data);
        Ok(())
    });
    server.run(router).expect("Starting server failed");
}
