use std::fs::{self, File};

use http_server::{router::{Body, HttpResponse, Router}, HttpServer};
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
struct Example {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum FileType {
    Directory,
    File,
}

#[derive(Debug, Serialize, Deserialize)]
struct Files {
    path: String,
    file_type: FileType,
}

fn main() {
    let server = HttpServer {
        port: 7878,
        threads: 4,
    };
    let mut router = Router::new();

    router.add_route("/{file}?","GET", |_, params| {
        let file_name = match params.get("file") {
            Some(file) => file,
            None => "index.html",
            
        };

        let contents = fs::read_to_string(file_name)?;
        Ok(HttpResponse::new(Body::Text(contents), Some(mime_guess::from_path(file_name).first_or_text_plain().to_string()), 200))
    });
    router.add_route("/api/error", "GET", |data, _| {
        println!("Request to other path with data {}",data.unwrap());
        Ok(HttpResponse::new(Body::Text("Error occured".to_string()),Some(String::from("text/plain")), 500))
    });
    router.add_route("/api/files", "GET", |_, params| {
        let file_name = format!("./{}", params.get("path").ok_or("Missing required query param")?);
        let file = File::open(&file_name)?;
        Ok(HttpResponse::new(Body::File(file), Some(mime_guess::from_path(file_name).first_or_octet_stream().to_string()), 200))
    
    });

    router.add_route("/api/directory", "GET", |_, params| {
        //println!("Request to directory path with query param: {:?}", params.unwrap());
        Ok(HttpResponse::new(Body::Json(list_directory(&format!("./{}", params.get("path").unwrap_or_else(|| {&""})))?), None, 200))
    });

    router.add_route("/api/json", "GET", |_, _ | {
        Ok(HttpResponse::new(Body::Json(serde_json::to_value(Example {message: String::from("Hello")})?), None, 200))
    
    });
    server.run(router).expect("Starting server failed");
}


fn list_directory(path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let paths = fs::read_dir(path)?;

    let mut path_info = Vec::new();
    for path in paths {
        let path = path?;
        let file = Files {
            path: format!("{}", path.path().display()),
            file_type: if path.path().is_dir() { FileType::Directory } else { FileType::File },
        };
        path_info.push(file);
    }

    let v = serde_json::to_value(&path_info)?;
    
    Ok(v)
}