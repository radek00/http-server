use std::fmt::format;
use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};

use router::Router;

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

        for stream in listener.incoming() {
            let stream = stream?;
            handle_connection(stream, &router);
            // pool.execute( | | {
            // })?;
        }
        Ok(())
    }
}

fn handle_connection(mut stream: TcpStream, router: &Router){
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let http_parts: Vec<&str> = request.split("\r\n\r\n").collect();
    let request_lines: Vec<&str> = http_parts[0].lines().collect();

    let http_method: Vec<&str> = request_lines[0].split_whitespace().collect();
    let (method, path, _version) = (http_method[0], http_method[1], http_method[2]);

    let body = if http_parts.len() > 1 { http_parts[1] } else { "" };

    router.route(path, method, body).unwrap();
}