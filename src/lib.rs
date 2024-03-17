use std::net::{SocketAddr, TcpListener, TcpStream};

mod thread_pool;

pub struct HttpServer {
    pub port: u16,
    pub threads: usize,
}

impl HttpServer {
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Server is running on port {}", self.port);
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], self.port)))?;
        let pool = thread_pool::ThreadPool::build(self.threads)?;

        for stream in listener.incoming() {
            let stream = stream?;
            pool.execute(|| {
                handle_connection(stream);
            })?;
        }
        Ok(())
    }
}

fn handle_connection(stream: TcpStream) {
    
}