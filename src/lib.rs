
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use router::{Body, HttpResponse, Router};
use serde_json::json;

mod thread_pool;
pub mod router;
pub mod static_files;
pub mod utils;

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

fn handle_connection(stream: &TcpStream, router: Arc<Mutex<Router>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stream);
    
    let mut request = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        request.push_str(&line);
        if line == "\r\n" {
            break;
        }
    }

    let http_parts: Vec<&str> = request.split("\r\n\r\n").collect();
    let request_lines: Vec<&str> = http_parts[0].lines().collect();

    let http_method: Vec<&str> = request_lines[0].split_whitespace().collect();
    let (method, path, _version) = (http_method[0], http_method[1], http_method[2]);

    let mut headers = std::collections::HashMap::new();
    for line in &request_lines[1..] {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            headers.insert(parts[0].trim(), parts[1].trim());
        }
    }

    let mut buffer = Vec::new();
    let body;
    match headers.get("Content-Type") {
        Some(content_type) => {
            if content_type.contains("multipart/form-data") {
                let response = handle_multipart_file_upload(&content_type, &headers, &mut reader, &path)?;
                return write_response(&response, stream)
            } else {
                body = parse_body(&headers, &mut reader, &mut buffer)?;
            }
        }
        None => {
            body = parse_body(&headers, &mut reader, &mut buffer)?;
        }
    }

    let response = router.lock().unwrap().route(path, method, body.as_deref())?;
    write_response(&response, &stream)?;
    
    Ok(())
}

fn parse_body<'a>(headers: &'a HashMap<&'a str, &'a str>, reader: &'a mut BufReader<&'a TcpStream>, mut buffer: &'a mut Vec<u8>) -> Result<Option<Cow<'a, str>>, Box<dyn std::error::Error>> {
    match headers.get("Content-Length") {
        Some(content_length) => {
            let content_length = content_length.parse::<usize>()?;
            let mut body_reader = reader.take(content_length.try_into()?);    
            body_reader.read_to_end(&mut buffer)?;
            let body = String::from_utf8_lossy(&buffer[..]);
            Ok(Some(body))
        },
        None => Ok(None),
    }
}

fn handle_multipart_file_upload(content_type: &str, headers: &HashMap<&str, &str>, reader: &mut BufReader<&TcpStream>, path: &str) -> Result<HttpResponse, Box<dyn std::error::Error>>  {
    let idx = content_type.find("boundary=").ok_or("Missing multipart boundary")?;
    let boundary = format!("--{}", &content_type[(idx + "boundary=".len())..]);
    let mut multipart_headers = HashMap::new();
    let mut header_size = 0;

    //read headers
    loop {
        let mut line = String::new();
        header_size += reader.read_line(&mut line)?;
        if line.trim() == boundary {
            continue;
        }
        if line == "\r\n" {
            break;
        }

        let parts: Vec<&str> = line.trim().split(':').collect();
        multipart_headers.insert(parts[0].to_owned(), parts[1].to_owned());
    }

    //get file name from content disposition and form target path
    let content_disposition = multipart_headers.get("Content-Disposition").ok_or("Missing content disposition")?;
    let filename = content_disposition
        .split("filename=\"")
        .nth(1)
        .and_then(|s| s.split("\"").next())
        .ok_or("Error parsing file name")?;
    let target_path = format!("{}/{}", path, filename);

    //calculate file size based on whole content length so that reading the stream can be stopped
    let mut file = File::create(target_path)?;
    let content_length = headers.get("Content-Length").ok_or("Missing content length")?.parse::<usize>()?;
    let file_bytes = content_length - boundary.len() - header_size - 6;

    //take only the file length from the main buf reader
    let mut limited_reader = reader.take(file_bytes.try_into()?);

    //copy streams
    io::copy(&mut  limited_reader, &mut  file)?;

    let response = HttpResponse::new(Body::Text(format!("File {} uploaded successfully.", filename)), Some(String::from("text/plain")), 200);
    Ok(response)
}