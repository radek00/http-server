use api_error::ApiError;
use http_parse_error::HttpParseError;
use include_dir::{include_dir, Dir};
use logger::Logger;
use native_tls::{Identity, TlsAcceptor};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use termcolor::Color;

mod errors;
mod http_response;
mod logger;
mod router;
mod thread_pool;

pub use errors::*;
pub use http_response::*;
pub use router::*;

pub static STATIC_FILES: Dir<'_> = include_dir!("src/dist");

pub trait ReadWrite: Read + Write + Send + 'static {}

impl<T: Read + Write + Send + 'static> ReadWrite for T {}

struct NetworkStream {
    delegate: Option<Box<dyn ReadWrite>>,
    tls_acceptor: Option<TlsAcceptor>,
}

impl NetworkStream {
    pub fn new(
        cert_path: Option<&PathBuf>,
        cert_pass: Option<&String>,
    ) -> Result<NetworkStream, Box<dyn std::error::Error>> {
        match &cert_path {
            Some(path) => {
                let identity_bytes = fs::read(path)?;

                let identity = Identity::from_pkcs12(&identity_bytes, cert_pass.unwrap())?;

                let tls_acceptor = TlsAcceptor::new(identity)?;

                Ok(NetworkStream {
                    tls_acceptor: Some(tls_acceptor),
                    delegate: None,
                })
            }
            None => Ok(NetworkStream {
                tls_acceptor: None,
                delegate: None,
            }),
        }
    }
    pub fn get_stream(
        &mut self,
        stream: TcpStream,
    ) -> Result<&mut NetworkStream, Box<dyn std::error::Error>> {
        match &self.tls_acceptor {
            Some(acceptor) => {
                let tls_stream = acceptor.accept(stream)?;
                self.delegate = Some(Box::new(tls_stream));
                Ok(self)
            }
            None => {
                self.delegate = Some(Box::new(stream));
                Ok(self)
            }
        }
    }
}

pub struct HttpServer {
    port: u16,
    threads: usize,
    cert_path: Option<PathBuf>,
    cert_pass: Option<String>,
    router: Router,
    logger: Option<Arc<Logger>>,
    bind_address: IpAddr,
}

impl HttpServer {
    pub fn build(
        port: u16,
        threads: usize,
        cert_path: Option<PathBuf>,
        cert_pass: Option<String>,
        bind_address: IpAddr,
    ) -> HttpServer {
        HttpServer {
            port,
            threads,
            cert_path,
            cert_pass,
            router: Router::new(),
            logger: None,
            bind_address,
        }
    }
    pub fn with_logger(mut self) -> Self {
        self.logger = Some(Arc::new(Logger::new()));
        self.router = self
            .router
            .with_logger(Some(Arc::clone(self.logger.as_ref().unwrap())));
        self
    }

    pub fn add_routes<F>(mut self, routes: F) -> Self
    where
        F: Fn(&mut Router) + Send + Sync + 'static,
    {
        routes(&mut self.router);
        self
    }

    pub fn with_cors_policy(mut self, policy: Cors) -> Self {
        self.router = self.router.with_cors(policy);
        self
    }
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        self.print_server_info();
        let listener = TcpListener::bind(SocketAddr::from((self.bind_address, self.port)))?;
        let pool = thread_pool::ThreadPool::build(self.threads)?;

        let arc_router = Arc::new(self.router);
        let mut network_stream =
            NetworkStream::new(self.cert_path.as_ref(), self.cert_pass.as_ref())?;
        for stream in listener.incoming() {
            let stream = stream?;
            let peer_addr = stream.peer_addr()?;
            let Ok(stream) = network_stream.get_stream(stream) else {
                continue;
            };
            let mut stream = stream.delegate.take().unwrap();

            let router_clone = Arc::clone(&arc_router);
            let logger_clone = self.logger.clone();

            pool.execute(move || {
                handle_connection(&mut stream, &router_clone, peer_addr.ip())
                    .unwrap_or_else(|err| {
                        if let (Some(method), Some(path)) = (&err.method, &err.path) {
                            router_clone
                                .log_response(
                                    err.error_response.status_code,
                                    path,
                                    method,
                                    peer_addr.ip(),
                                )
                                .unwrap();
                        }

                        err.error_response
                    })
                    .write_response(&mut stream)
                    .unwrap_or_else(|err| {
                        if let Some(logger) = logger_clone {
                            logger
                                .log_stderr("Error: {}", vec![(err.to_string(), Some(Color::Red))])
                                .unwrap();
                        }
                    });
            })?;
        }
        Ok(())
    }
    fn print_server_info(&self) {
        if let Some(logger) = &self.logger {
            let https = match self.cert_path {
                Some(_) => String::from("Enabled"),
                None => String::from("Disabled"),
            };
            let cors = if self.router.hsa_cors() {
                String::from("Enabled")
            } else {
                String::from("Disabled")
            };

            logger.log_stdout(
                r#"

 ========================================================================================================
 
   _____ _                 _        _    _ _______ _______ _____     _____                          
  / ____(_)               | |      | |  | |__   __|__   __|  __ \   / ____|                         
 | (___  _ _ __ ___  _ __ | | ___  | |__| |  | |     | |  | |__) | | (___   ___ _ ____   _____ _ __ 
  \___ \| | '_ ` _ \| '_ \| |/ _ \ |  __  |  | |     | |  |  ___/   \___ \ / _ \ '__\ \ / / _ \ '__|
  ____) | | | | | | | |_) | |  __/ | |  | |  | |     | |  | |       ____) |  __/ |   \ V /  __/ |   
 |_____/|_|_| |_| |_| .__/|_|\___| |_|  |_|  |_|     |_|  |_|      |_____/ \___|_|    \_/ \___|_|   
                    | |                                                                             
                    |_|                                                                             

=========================================================================================================

Port: {}
Threads: {}
HTTPS: {}
CORS: {}

====================
Logs:"#,
                vec![
                    (self.port.to_string(), Some(Color::Yellow)),
                    (self.threads.to_string(), Some(Color::Yellow)),
                    (https, Some(Color::Yellow)),
                    (cors, Some(Color::Yellow)),
                ],
            )
            .unwrap();
        }
    }
}

fn parse_http<'a>(
    reader: &mut BufReader<&mut Box<dyn ReadWrite>>,
    request_string: &'a mut String,
) -> Result<(&'a str, &'a str, HashMap<&'a str, &'a str>), HttpParseError> {
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        request_string.push_str(&line);
        if line == "\r\n" {
            break;
        }
    }
    let http_parts: Vec<&str> = request_string.split("\r\n\r\n").collect();
    let request_lines: Vec<&str> = http_parts
        .first()
        .ok_or(HttpParseError::default())?
        .lines()
        .collect();

    let http_method: Vec<&str> = request_lines
        .first()
        .ok_or(HttpParseError::default())?
        .split_whitespace()
        .collect();

    if http_method.len() < 3 {
        return Err(HttpParseError::default());
    }

    let (method, path, _version) = (http_method[0], http_method[1], http_method[2]);

    let mut headers = std::collections::HashMap::new();
    for line in &request_lines[1..] {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            headers.insert(
                *parts.first().ok_or(HttpParseError::default())?,
                parts.get(1).ok_or(HttpParseError::default())?.trim(),
            );
        }
    }

    Ok((method, path, headers))
}

fn handle_connection(
    stream: &mut Box<dyn ReadWrite>,
    router: &Arc<Router>,
    peer_addr: IpAddr,
) -> Result<HttpResponse, ApiError> {
    let mut reader = BufReader::new(&mut *stream);

    let mut request = String::new();
    let (method, path, headers) = parse_http(&mut reader, &mut request)?;

    let mut buffer = Vec::new();
    let body;

    match headers.get("Content-Type") {
        Some(content_type) => {
            if content_type.contains("multipart/form-data") {
                let path = headers.get("Path").unwrap();
                let response =
                    handle_multipart_file_upload(content_type, &headers, &mut reader, path)
                        .map_err(|err| {
                            ApiError::new_with_html(400, format!("File upload error: {}", err))
                        })?;
                return Ok(response);
            } else {
                body = parse_body(&headers, reader, &mut buffer)?;
            }
        }
        None => {
            body = parse_body(&headers, reader, &mut buffer)?;
        }
    }

    let response = router.route(path, method, body.as_deref(), peer_addr)?;
    Ok(response)
}

fn parse_body<'a>(
    headers: &'a HashMap<&'a str, &'a str>,
    reader: BufReader<&mut Box<dyn ReadWrite>>,
    buffer: &'a mut Vec<u8>,
) -> Result<Option<Cow<'a, str>>, Box<dyn std::error::Error>> {
    match headers.get("Content-Length") {
        Some(content_length) => {
            let content_length = content_length.parse::<usize>()?;
            let mut body_reader = reader.take(content_length.try_into()?);
            body_reader.read_to_end(buffer)?;
            let body = String::from_utf8_lossy(&buffer[..]);
            Ok(Some(body))
        }
        None => Ok(None),
    }
}

fn handle_multipart_file_upload(
    content_type: &str,
    headers: &HashMap<&str, &str>,
    reader: &mut BufReader<&mut Box<dyn ReadWrite>>,
    path: &str,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let idx = content_type
        .find("boundary=")
        .ok_or("Missing multipart boundary")?;
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

        let parts: Vec<&str> = line.trim().split(':').map(|s| s.trim()).collect();
        if parts.len() < 2 {
            return Err("Error parsing multipart request".into());
        }
        multipart_headers.insert(parts[0].to_owned(), parts[1].to_owned());
    }

    //get file name from content disposition and form target path
    let content_disposition = multipart_headers
        .get("Content-Disposition")
        .ok_or("Missing content disposition")?;
    let filename = content_disposition
        .split("filename=\"")
        .nth(1)
        .and_then(|s| s.split('\"').next())
        .ok_or("Error parsing file name")?;
    let mut target_path = PathBuf::from("./").canonicalize()?.join(path);
    target_path.push(filename);

    let current_dir = std::env::current_dir()?;
    if !target_path.starts_with(current_dir) {
        return Err("Only paths relative to the current directory are allowed".into());
    }

    //calculate file size based on whole content length so that reading the stream can be stopped
    let mut file = File::create(target_path)?;
    let content_length = headers
        .get("Content-Length")
        .ok_or("Missing content length")?
        .parse::<usize>()?;
    let file_bytes = content_length - boundary.len() - header_size - 6;

    //take only the file length from the main buf reader
    let mut limited_reader = reader.take(file_bytes.try_into()?);

    //copy streams
    io::copy(&mut limited_reader, &mut file)?;

    let response = HttpResponse::new(
        Some(Body::Text(format!(
            "File {} uploaded successfully.",
            filename
        ))),
        Some(String::from("text/plain")),
        200,
    );
    Ok(response)
}
