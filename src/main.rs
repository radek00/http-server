use std::fs;

use http_server::{router::{Body, HttpResponse, Router}, HttpServer};
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
struct Example {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Files {
    path: String,
    file_type: String,
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
    router.add_route("/error", "GET", |data, _| {
        println!("Request to other path with data {}",data.unwrap());
        Ok(HttpResponse::new(Body::Text("Error occured".to_string()),Some(String::from("text/plain")), 500))
    });
    router.add_route("/file/{name}", "GET", |data, params| {
        println!("Request to file path with data {}, dynamic route {} and query param: {}",data.unwrap(), params.unwrap().get("name").unwrap(), params.unwrap().get("test").unwrap());
        Ok(HttpResponse::new(Body::Text("File found".to_string()) ,Some(String::from("text/plain")), 200))
    
    });

    router.add_route("/directory", "GET", |_, params| {
        Ok(HttpResponse::new(Body::Json(list_directory(params.unwrap().get("path").unwrap())), None, 200))
    });

    router.add_route("/json", "GET", |_, _ | {
        Ok(HttpResponse::new(Body::Json(serde_json::to_value(Example {message: String::from("Hello")}).unwrap()), None, 200))
    
    });
    server.run(router).expect("Starting server failed");
}


fn list_directory(path: &str) -> serde_json::Value {
    let paths = fs::read_dir(path).unwrap();

    let mut path_info = Vec::new();
    for path in paths {
        let path = path.unwrap();
        let file = Files {
            path: format!("{}", path.path().display()),
            file_type: if path.path().is_dir() { "directory".to_string() } else { "file".to_string() },
        };
        path_info.push(file);
    }

    let v = serde_json::to_value(&path_info).unwrap();
    
    v
}