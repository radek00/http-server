use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use multipart::server::{Multipart, SaveResult};
use serde_json::json;

use router::{Body, HttpResponse, Router};

mod thread_pool;
pub mod router;
pub mod static_files;

// #[derive(Debug)]
// enum HttpMethod {
//     GET,
//     POST,
//     PUT,
//     DELETE,
//     PATCH,
//     OPTIONS,
//     HEAD,
//     TRACE,
//     CONNECT,
// }


fn write_response(response: &HttpResponse, mut stream: &TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let body = match &response.body {
        Body::Text(text) => text.clone(),
        Body::Json(json) => json.to_string(),
        Body::StaticFile(file, _) => {
            let headers = format!(
                "HTTP/1.1 {}\r\n\
                Content-Type: {}\r\n\
                Connection: keep-alive\r\n\
                Server: RustHttpServer/1.0\r\n\
                \r\n", response.status_code, response.content_type
            );
            stream.write_all(headers.as_bytes())?;
            stream.write_all(file)?;
            return Ok(());
        },
        Body::FileStream(file, name) => {
            let headers = format!(
                "HTTP/1.1 {}\r\n\
                Content-Type: {}\r\n\
                Content-Disposition: attachment; filename=\"{}\"\r\n\
                Connection: keep-alive\r\n\
                Server: RustHttpServer/1.0\r\n\
                \r\n", response.status_code, response.content_type, name
            );
            stream.write_all(headers.as_bytes())?;
            let mut reader = BufReader::new(file);
            io::copy(&mut reader, &mut stream)?;
            return Ok(());
        }
    };

    let response = format!(
        "HTTP/1.1 {}\r\n\
        Content-Type: {}\r\n\
        Content-Length: {}\r\n\
        Connection: keep-alive\r\n\
        Server: RustHttpServer/1.0\r\n\
        \r\n\
        {}",
        response.status_code,
        response.content_type,
        body.len(),
        body
    );

    stream.write_all(response.as_bytes())?;

    Ok(())
}

pub struct HttpServer {
    pub port: u16,
    pub threads: usize,
}

impl HttpServer {
    pub fn run(&self, router: Router) -> Result<(), Box<dyn std::error::Error>> {
        println!("Server is running on port {}", self.port);
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], self.port)))?;
        let pool = thread_pool::ThreadPool::build(self.threads)?;

        let arc_router = Arc::new(Mutex::new(router));
        for stream in listener.incoming() {
            let stream = stream?;
            let router_clone = Arc::clone(&arc_router);
            pool.execute( move | | {
                handle_connection(&stream, router_clone).unwrap_or_else(|err| {
                    eprintln!("{}", err);
                    let error_message = json!({
                        "error": format!("{}", err)
                    });
                    let error_response = HttpResponse::new(Body::Json(error_message), None, 500);
                    write_response(&error_response, &stream).unwrap_or_else(|err| {
                        eprintln!("{}", err);                    
                    })
                });
            })?;
        }
        Ok(())
    }
}

fn handle_connection(mut stream: &TcpStream, router: Arc<Mutex<Router>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;

    let request = String::from_utf8_lossy(&buffer[..]);
    let http_parts: Vec<&str> = request.split("\r\n\r\n").collect();
    let request_lines: Vec<&str> = http_parts[0].lines().collect();

    let http_method: Vec<&str> = request_lines[0].split_whitespace().collect();
    let (method, path, _version) = (http_method[0], http_method[1], http_method[2]);

    let body = if http_parts.len() > 1 { Some(http_parts[1]) } else { None };

    let mut headers = std::collections::HashMap::new();
    for line in &request_lines[1..] {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            headers.insert(parts[0].trim(), parts[1].trim());
        }
    }

    match headers.get("Content-Type") {
        Some(content_type) => {
            if content_type.contains("multipart/form-data") {
                let idx = content_type.find("boundary=").expect("no boundary");
                let boundary = &content_type[(idx + "boundary=".len())..];

                let mut multipart  = Multipart::with_body(Cursor::new(&buffer), boundary);
                // multipart.foreach_entry(|mut entry| {
                //     match entry.headers.name.to {
                //         "file" => {
                //             println!("file");
                //         }
                //     }
                // });
                match multipart.save().size_limit(u64::MAX).ignore_text().temp() {
                    SaveResult::Full(entries) => {
                        let files_fields = match entries.fields.get("file") {
                            Some(fields) => fields,
                            None => {
                                return Err(Box::from("Error"))
                            }
                        };

                        for field in files_fields {
                            let mut data = field.data.readable().unwrap();
                            let headers = &field.headers;
                            let mut target_path = format!("{}/{}", path, headers.filename.clone().unwrap());

                            // target_path.push(headers.filename.clone().unwrap());
                           std::fs::File::create(target_path)
                                .and_then(|mut file| io::copy(&mut data, &mut file))?;
                        }
                        return Ok(())
                    }
                    SaveResult::Partial(_, reason) => {
                        println!("Failed to save multipart form data");
                    }
                    SaveResult::Error(err) => {
                        println!("Error saving multipart form data: {}", err);
                    
                    }
                }
                // println!("Content-Type: {}", content_type);
                // let file = File::create("file.exe")?;
                // let mut writer  = BufWriter::new(file);
                // io::copy(&mut stream, &mut writer)?;

            }
        }
        None => {
            println!("No Content-Type header found");
        }
    }

    let response = router.lock().unwrap().route(path, method, body)?;
    write_response(&response, &stream)?;
    
    Ok(())
}