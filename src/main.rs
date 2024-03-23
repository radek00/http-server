use http_server::{router::{HttpResponse, Router}, HttpServer};

fn main() {
    let server = HttpServer {
        port: 7878,
        threads: 4,
    };
    let mut router = Router::new();
    router.add_route("/","GET", |_, _| {
        println!("Request to root path");
        Ok(HttpResponse::new("Hello from root path".to_string(), None, 200))
    });
    router.add_route("/error", "GET", |data, params| {
        println!("Request to other path with data {} and params {}",data.unwrap(), params.unwrap());
        Ok(HttpResponse::new("Error occured".to_string(), None, 500))
    });
    router.add_route("/file/{name}", "GET", |data, params| {
        println!("Request to file path with data {} and params {}",data.unwrap(), params.unwrap());
        Ok(HttpResponse::new("File found".to_string(), None, 200))
    
    });

    router.add_route("/json", "GET", |_, _ | {
        Ok(HttpResponse::new("JSON found".to_string(), None, 200))
    
    });
    server.run(router).expect("Starting server failed");
}
