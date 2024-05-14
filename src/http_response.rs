use std::{
    fs::File,
    io::{self, BufReader},
};

use crate::ReadWrite;

pub enum Body {
    Text(String),
    Json(serde_json::Value),
    FileStream(File, String),
    StaticFile(&'static [u8], String),
}

pub struct HttpResponse {
    pub content_type: String,
    pub body: Body,
    pub status_code: u16,
}

impl HttpResponse {
    pub fn new(body: Body, content_type: Option<String>, status_code: u16) -> Self {
        HttpResponse {
            content_type: content_type.unwrap_or_else(|| "application/json".to_string()),
            body,
            status_code,
        }
    }
    pub fn write_response(
        self,
        mut stream: &mut Box<dyn ReadWrite>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let body = match &self.body {
            Body::Text(text) => text.clone(),
            Body::Json(json) => json.to_string(),
            Body::StaticFile(file, _) => {
                let headers = format!(
                    "HTTP/1.1 {}\r\n\
                    Content-Type: {}\r\n\
                    Connection: keep-alive\r\n\
                    Server: RustHttpServer/1.0\r\n\
                    \r\n",
                    self.status_code, self.content_type
                );
                stream.write_all(headers.as_bytes())?;
                stream.write_all(file)?;
                return Ok(());
            }
            Body::FileStream(file, name) => {
                let headers = format!(
                    "HTTP/1.1 {}\r\n\
                    Content-Type: {}\r\n\
                    Content-Disposition: attachment; filename=\"{}\"\r\n\
                    Connection: keep-alive\r\n\
                    Server: RustHttpServer/1.0\r\n\
                    \r\n",
                    self.status_code, self.content_type, name
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
            self.status_code,
            self.content_type,
            body.len(),
            body
        );

        stream.write_all(response.as_bytes())?;

        Ok(())
    }
}
