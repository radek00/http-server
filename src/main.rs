use std::{env, fs::{self, File}};

use chrono::{DateTime, Utc};
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
    name: String,
    file_type: FileType,
    last_modified: String,
    size: u64,
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
        let file_path = params.get("path").ok_or("Missing path parameter")?;
        let file = File::open(&file_path)?;
        Ok(HttpResponse::new(Body::File(file, file_path.split('/').last().ok_or("Path error")?.to_string()), Some(mime_guess::from_path(&file_path).first_or_octet_stream().to_string()), 200))
    
    });

    router.add_route("/api/directory", "GET", |_, params| {
        //println!("Request to directory path with query param: {:?}", params.unwrap());
        Ok(HttpResponse::new(Body::Json(list_directory(params.get("path").ok_or("Missing path parameter")?)?), None, 200))
    });

    router.add_route("/api/path", "GET", |_,_| {
        Ok(HttpResponse::new(Body::Text(env::current_dir()?.to_string_lossy().to_string()), Some(String::from("text/plain")), 200))
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
        let system_time:  DateTime<Utc> = path.metadata()?.modified()?.into();

        let file = Files {
            name: path.file_name().into_string().unwrap(),
            path: fs::canonicalize(path.path())?.to_string_lossy().into_owned(),
            file_type: if path.path().is_dir() { FileType::Directory } else { FileType::File },
            last_modified: system_time.format("%d/%m/%Y %T").to_string(),
            size: path.metadata()?.len(),
        };
        path_info.push(file);
    }

    let v = serde_json::to_value(path_info)?;
    
    Ok(v)
}