use std::fmt;

use crate::{Body, HttpResponse};

#[derive(Debug)]
pub struct ApiError {
    pub error_response: HttpResponse,
    pub method: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug)]
pub struct HttpParseError {
    pub message: String,
}

impl fmt::Display for HttpParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for HttpParseError {}

impl From<std::io::Error> for HttpParseError {
    fn from(error: std::io::Error) -> Self {
        HttpParseError {
            message: error.to_string(),
        }
    }
}

impl ApiError {
    pub fn new(code: u16, message: String) -> Self {
        ApiError {
            error_response: format_error(code, message),
            method: None,
            path: None,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ApiError {}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        println!("Error io: {}", error);
        ApiError::new(500, error.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for ApiError {
    fn from(error: Box<dyn std::error::Error>) -> ApiError {
        println!("Error box: {}", error);
        ApiError::new(500, error.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        println!("Error notmal: {}", error);
        ApiError::new(500, error.to_string())
    }
}

impl From<&str> for ApiError {
    fn from(error: &str) -> Self {
        ApiError::new(500, error.to_string())
    }
}

impl From<HttpParseError> for ApiError {
    fn from(error: HttpParseError) -> Self {
        ApiError::new(
            500,
            format!("Error parsing HTTP request: {}", error.message),
        )
    }
}

fn format_error(error_code: u16, message: String) -> HttpResponse {
    let html = format!(
        "<!DOCTYPE html>
    <html lang=\"en\">
    <head>
        <meta charset=\"UTF-8\">
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
        <title>Error</title>
        <style>
        body {{
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            font-family: Arial, sans-serif;
        }}
        .error-container {{
            text-align: center;
        }}
        .error-container h1 {{
            font-size: 3em;
            color: #ff0000;
        }}
        .error-container p {{
            font-size: 1.5em;
        }}
    </style>
    </head>

    <body>
        <div class=\"error-container\">
            <h1>{} {}</h1>
            <p>{}</p>
        </div>
    </body>
    </html>",
        error_code, "Not Found", message
    );
    HttpResponse::new(
        Body::Text(html),
        Some(String::from("text/html")),
        error_code,
    )
}
