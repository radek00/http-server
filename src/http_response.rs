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
    FileStream(File, String),
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
        mut stream: &mut Box<dyn ReadWrite>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut base_headers = format!(
            "HTTP/1.1 {}\r\n\
            Content-Type: {}\r\n\
            Connection: keep-alive\r\n\
            Server: RustHttpServer/1.0\r\n\
            ",
            self.status_code, self.content_type
        );

            //check if compression is enabled
    //check if accept-encoding header is present
    //if gzip is supported, compress the response
    //if not, return the response as is
    //set the content-encoding header to gzip
    //set Vary header to Accept-Encoding

        self.headers.iter().for_each(|(key, value)| {
            base_headers.push_str(&format!("{}: {}\r\n", key, value));
        });

        if let Some(body) = self.body {
            if true {
                //if compression is enabled
                base_headers.push_str("Content-Encoding: gzip\r\n");
                base_headers.push_str("Vary: Accept-Encoding\r\n");
    
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

                match body {
                    Body::Text(text) => {
                        encoder.write_all(text.as_bytes())?
                    },
                    Body::Json(json) => {
                        encoder.write_all(json.to_string().as_bytes())?
                    },
                    Body::StaticFile(file, _) => {
                        encoder.write_all(file)?;
                        let encoded = encoder.finish()?;
                        base_headers.push_str(&format!("Content-Length: {}\r\n", encoded.len()));
                        base_headers.push_str("\r\n");
                        stream.write_all(base_headers.as_bytes())?;
                        stream.write_all(&encoded)?;
                        return Ok(());
                    }
                    Body::FileStream(file, name) => {
                        base_headers.push_str(&format!(
                            "Content-Disposition: attachment; filename=\"{}\"\r\n\
                        \rn",
                            name
                        ));
                        stream.write_all(base_headers.as_bytes())?;
                        let mut reader = BufReader::new(file);
                        io::copy(&mut reader, &mut stream)?;
                        return Ok(());
                    }
                }

                let encoded = encoder.finish()?;
                base_headers.push_str(&format!(
                    "Content-Length: {}\r\n\
                \r\n\
                ",
                    encoded.len(),
                ));
                stream.write_all(base_headers.as_bytes())?;
                stream.write_all(&encoded)?;
            } else {
                let body = match body {
                    Body::Text(text) => text.clone(),
                    Body::Json(json) => json.to_string(),
                    Body::StaticFile(file, _) => {
                        base_headers.push_str(&format!("Content-Length: {}\r\n", file.len()));
                        base_headers.push_str("\r\n");
                        stream.write_all(base_headers.as_bytes())?;
                        stream.write_all(file)?;
                        return Ok(());
                    }
                    Body::FileStream(file, name) => {
                        base_headers.push_str(&format!(
                            "Content-Disposition: attachment; filename=\"{}\"\r\n\
                        \rn",
                            name
                        ));
                        stream.write_all(base_headers.as_bytes())?;
                        let mut reader = BufReader::new(file);
                        io::copy(&mut reader, &mut stream)?;
                        return Ok(());
                    }
                };
                base_headers.push_str(&format!(
                    "Content-Length: {}\r\n\
                \r\n\
                {}",
                    body.len(),
                    body
                ));
                stream.write_all(base_headers.as_bytes())?;
            }
    
            }

        Ok(())
    }
    pub fn add_response_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }
}
