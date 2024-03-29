use std::io::{self, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use serde_json::json;

use router::{Body, HttpResponse, Router};

mod thread_pool;
pub mod router;

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
                handle_connection(stream, router_clone);
            })?;
        }
        Ok(())
    }
}

fn handle_connection(mut stream: TcpStream, router: Arc<Mutex<Router>>){
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let http_parts: Vec<&str> = request.split("\r\n\r\n").collect();
    let request_lines: Vec<&str> = http_parts[0].lines().collect();

    let http_method: Vec<&str> = request_lines[0].split_whitespace().collect();
    let (method, path, _version) = (http_method[0], http_method[1], http_method[2]);

    let body = if http_parts.len() > 1 { Some(http_parts[1]) } else { None };

    let response = router.lock().unwrap().route(path, method, body).unwrap_or_else(| err| {
        let error_message = json!({
            "error": format!("{}", err)
        });
        HttpResponse::new(Body::Json(error_message), None, 500)
    });

    let mut write_response = |body_string: &str| {
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
            body_string.len(),
            body_string
        );
    
        stream.write_all(response.as_bytes()).unwrap();
    };
    
    match &response.body {
        Body::Text(text) => {
            write_response(text);
        }
        Body::Json(json) => {
            let json_string = json.to_string();
            write_response(&json_string);
        }
        Body::File(file) => {
            let headers = format!(
                "HTTP/1.1 200 OK\r\n\
                Content-Type: {}\r\n\
                Connection: keep-alive\r\n\
                Server: RustHttpServer/1.0\r\n\
                \r\n", response.content_type
            );

            stream.write_all(headers.as_bytes()).unwrap();
    
            let mut reader = BufReader::new(file);
            io::copy(&mut reader, &mut stream).unwrap();
        }
    }
}