use std::{
    fs::File,
    io::{self, BufReader, Write},
};

use flate2::write::GzEncoder;
use flate2::Compression;

use crate::ReadWrite;

#[derive(Debug)]
pub enum Body {
    Text(String),
    Json(serde_json::Value),
    DownloadStream(File, String),
    FileStream(File),
    StaticFile(&'static [u8], String),
}

#[derive(Debug)]
pub struct HttpResponse {
    pub content_type: String,
    pub body: Option<Body>,
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
}

impl HttpResponse {
    pub fn new(body: Option<Body>, content_type: Option<String>, status_code: u16) -> Self {
        HttpResponse {
            content_type: content_type.unwrap_or_else(|| "application/json".to_string()),
            body,
            status_code,
            headers: Vec::new(),
        }
    }
    pub fn write_response(
        self,
        stream: &mut Box<dyn ReadWrite>,
        compress: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut base_headers = format!(
            "HTTP/1.1 {}\r\n\
            Content-Type: {}\r\n\
            Connection: keep-alive\r\n\
            Server: RustHttpServer/1.0\r\n\
            ",
            self.status_code, self.content_type
        );

        self.headers.iter().for_each(|(key, value)| {
            base_headers.push_str(&format!("{}: {}\r\n", key, value));
        });

        if let Some(body) = self.body {
            return match (body, compress) {
                (Body::DownloadStream(file, name), _) => {
                    handle_file_stream(file, Some(name), base_headers, stream, true)
                }
                (Body::FileStream(file), true) => {
                    handle_compressed_file_stream(file, base_headers, stream)
                }
                (Body::FileStream(file), false) => {
                    handle_file_stream(file, None, base_headers, stream, false)
                }
                (Body::Text(text), should_compress) => {
                    write_buffered_body(base_headers, text.as_bytes(), should_compress, stream)
                }
                (Body::Json(json), should_compress) => {
                    let serialized = json.to_string();
                    write_buffered_body(
                        base_headers,
                        serialized.as_bytes(),
                        should_compress,
                        stream,
                    )
                }
                (Body::StaticFile(file, _), should_compress) => {
                    write_buffered_body(base_headers, file, should_compress, stream)
                }
            };
        }

        Ok(())
    }
    pub fn add_response_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }
}

fn handle_file_stream(
    file: File,
    mut name: Option<String>,
    mut headers: String,
    mut stream: &mut Box<dyn ReadWrite>,
    is_attachment: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = file.metadata()?;
    let file_size = metadata.len();

    headers.push_str(&format!("Content-Length: {}\r\n", file_size));

    if is_attachment {
        headers.push_str(&format!(
            "Content-Disposition: attachment; filename=\"{}\"\r\n",
            name.take().unwrap()
        ));
    }
    headers.push_str("\r\n");

    stream.write_all(headers.as_bytes())?;
    let mut reader = BufReader::new(file);
    io::copy(&mut reader, &mut stream)?;
    Ok(())
}

fn handle_compressed_file_stream(
    file: File,
    mut headers: String,
    stream: &mut Box<dyn ReadWrite>,
) -> Result<(), Box<dyn std::error::Error>> {
    if headers.contains("Connection: keep-alive") {
        headers = headers.replace("Connection: keep-alive", "Connection: close");
    }
    headers.push_str("Content-Encoding: gzip\r\n");
    headers.push_str("Vary: Accept-Encoding\r\n");
    headers.push_str("\r\n");

    stream.write_all(headers.as_bytes())?;

    let mut encoder = GzEncoder::new(stream, Compression::default());
    let mut reader = BufReader::new(file);
    println!("headers: {}", headers);
    io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?;

    Ok(())
}

fn write_buffered_body(
    mut headers: String,
    body: &[u8],
    compress: bool,
    stream: &mut Box<dyn ReadWrite>,
) -> Result<(), Box<dyn std::error::Error>> {
    if compress {
        headers.push_str("Content-Encoding: gzip\r\n");
        headers.push_str("Vary: Accept-Encoding\r\n");

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(body)?;
        let encoded = encoder.finish()?;

        headers.push_str(&format!("Content-Length: {}\r\n", encoded.len()));
        headers.push_str("\r\n");
        stream.write_all(headers.as_bytes())?;
        stream.write_all(&encoded)?;
        return Ok(());
    }

    headers.push_str(&format!("Content-Length: {}\r\n", body.len()));
    headers.push_str("\r\n");
    stream.write_all(headers.as_bytes())?;
    stream.write_all(body)?;

    Ok(())
}
