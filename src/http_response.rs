use std::{
    fs::File,
    io::{self, BufReader},
};

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
    pub body: Body,
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
}

impl HttpResponse {
    pub fn new(body: Body, content_type: Option<String>, status_code: u16) -> Self {
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

        self.headers.iter().for_each(|(key, value)| {
            base_headers.push_str(&format!("{}: {}\r\n", key, value));
        });
        let body = match &self.body {
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

        Ok(())
    }
    pub fn add_response_header(mut self, key: String, value: String) -> Self {
        self.headers.push((key, value));
        self
    }
}
