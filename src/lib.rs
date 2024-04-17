use std::io::{self, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
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
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let buffer_clone = buffer.clone();
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut request = httparse::Request::new(&mut headers);
    let res = request.parse(&mut buffer)?;
    let body = &buffer_clone[res.unwrap()..];

        // Calculate the size of the body
        //let body_size = buffer.len() - res.unwrap();

        // Create a new buffer of the appropriate size
        //let mut body_buffer = vec![0; body_size];
    
        // Read the body into the new buffer
    //stream.read(&mut buffer)?;
    // let http_parts: Vec<&str> = request.split("\r\n\r\n").collect();
    // let request_lines: Vec<&str> = http_parts[0].lines().collect();

    // let http_method: Vec<&str> = request_lines[0].split_whitespace().collect();
    // let (method, path, _version) = (http_method[0], http_method[1], http_method[2]);

    // let body = if http_parts.len() > 1 { Some(http_parts[1]) } else { None };

    let response = router.lock().unwrap().route(&request.path.unwrap(), &request.method.unwrap(), Some(&String::from_utf8_lossy(body)))?;
    write_response(&response, &stream)?;
    
    Ok(())
}