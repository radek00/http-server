use http_server::{router::{HttpResponse, Router}, HttpServer};

fn main() {
    let server = HttpServer {
        port: 7878,
        threads: 4,
    };
    let mut router = Router::new();
    router.add_route("/","GET", |data| {
        println!("Request to root path with data {}",data);
        Ok(HttpResponse::new("Hello from root path".to_string(), None, 200))
    });
    router.add_route("/error", "GET", |data| {
        println!("Request to other path with data {}",data);
        Ok(HttpResponse::new("Error occured".to_string(), None, 500))
    });
    server.run(router).expect("Starting server failed");
}
