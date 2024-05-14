use native_tls::{Identity, TlsAcceptor};
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

mod http_response;
mod router;
mod static_files;
mod thread_pool;

pub use http_response::*;
pub use router::*;
pub use static_files::*;

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

                let identity = Identity::from_pkcs12(&identity_bytes, cert_pass.unwrap().as_str())?;

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
    pub port: u16,
    pub threads: usize,
    pub cert_path: Option<PathBuf>,
    pub cert_pass: Option<String>,
}

impl HttpServer {
    pub fn build() -> HttpServer {
        let mut args = clap::Command::new("Simple HTTP Server")
            .version("1.0")
            .author("radek00")
            .about("Multi-threaded HTTP server. Features include dynamic routing, static file serving, and file upload/download handling.")
            .arg(clap::Arg::new("port")
                .short('p')
                .value_parser(clap::value_parser!(u16))
                .default_value("7878")
                .long("port")
                .help("Sets the port number"))
            .arg(clap::Arg::new("threads")
                .short('t')
                .value_parser(clap::value_parser!(usize))
                .default_value("4")
                .long("threads")
                .help("Sets the number of threads"))
            .arg(clap::Arg::new("cert")
                .short('c')
                .value_parser(clap::value_parser!(PathBuf))
                .required(false)
                .long("cert")
                .requires("certpass")
                .help("TLS/SSL certificate"))
            .arg(clap::Arg::new("certpass")
                .long("certpass")
                .help("TLS/SSL certificate password"))
            .get_matches();

        HttpServer {
            port: args.remove_one::<u16>("port").unwrap(),
            threads: args.remove_one::<usize>("threads").unwrap(),
            cert_path: args.remove_one::<PathBuf>("cert"),
            cert_pass: args.remove_one::<String>("certpass"),
        }
    }
    pub fn run(&self, router: Router) -> Result<(), Box<dyn std::error::Error>> {
        println!("Server is running on port {}", self.port);
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], self.port)))?;
        let pool = thread_pool::ThreadPool::build(self.threads)?;

        let arc_router = Arc::new(Mutex::new(router));
        let mut network_stream =
            NetworkStream::new(self.cert_path.as_ref(), self.cert_pass.as_ref())?;
        for stream in listener.incoming() {
            let mut stream = network_stream.get_stream(stream?)?.delegate.take().unwrap();
            let router_clone = Arc::clone(&arc_router);

            pool.execute(move || {
                handle_connection(&mut stream, router_clone).unwrap_or_else(|err| {
                    eprintln!("{}", err);
                    let error_message = json!({
                        "error": format!("{}", err)
                    });
                    HttpResponse::new(Body::Json(error_message), None, 500)
                        .write_response(&mut stream)
                        .unwrap_or_else(|err| {
                            eprintln!("{}", err);
                        });
                });
            })?;
        }
        Ok(())
    }
}

fn handle_connection(
    stream: &mut Box<dyn ReadWrite>,
    router: Arc<Mutex<Router>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(&mut *stream);

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
                return handle_multipart_file_upload(content_type, &headers, &mut reader, path)?
                    .write_response(stream);
            } else {
                body = parse_body(&headers, reader, &mut buffer)?;
            }
        }
        None => {
            body = parse_body(&headers, reader, &mut buffer)?;
        }
    }

    router
        .lock()
        .unwrap()
        .route(path, method, body.as_deref())?
        .write_response(stream)?;
    Ok(())
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

        let parts: Vec<&str> = line.trim().split(':').collect();
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
    let target_path = format!("{}/{}", path, filename);

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
        Body::Text(format!("File {} uploaded successfully.", filename)),
        Some(String::from("text/plain")),
        200,
    );
    Ok(response)
}
