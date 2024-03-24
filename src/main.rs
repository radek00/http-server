use http_server::{router::{Body, HttpResponse, Router}, HttpServer};
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
struct Example {
    message: String,
}

fn main() {
    let server = HttpServer {
        port: 7878,
        threads: 4,
    };
    let mut router = Router::new();
    router.add_route("/","GET", |_, _| {
        println!("Request to root path");
        Ok(HttpResponse::new(Body::Text("Hello from root path".to_string()), Some(String::from("text/plain")), 200))
    });
    router.add_route("/error", "GET", |data, params| {
        println!("Request to other path with data {} and params {}",data.unwrap(), params.unwrap());
        Ok(HttpResponse::new(Body::Text("Error occured".to_string()),Some(String::from("text/plain")), 500))
    });
    router.add_route("/file/{name}", "GET", |data, params| {
        println!("Request to file path with data {} and params {}",data.unwrap(), params.unwrap());
        Ok(HttpResponse::new(Body::Text("File found".to_string()) ,Some(String::from("text/plain")), 200))
    
    });

    router.add_route("/json", "GET", |_, _ | {
        Ok(HttpResponse::new(Body::Json(serde_json::to_value(Example {message: String::from("Hello")}).unwrap()), None, 200))
    
    });
    server.run(router).expect("Starting server failed");
}
