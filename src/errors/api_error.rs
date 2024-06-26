use std::fmt;

use crate::{http_parse_error::HttpParseError, Body, HttpResponse};

#[derive(Debug)]
pub struct ApiError {
    pub error_response: HttpResponse,
    pub method: Option<String>,
    pub path: Option<String>,
}

impl ApiError {
    pub fn new_with_html(code: u16, message: String) -> Self {
        ApiError {
            error_response: format_error(code, message),
            method: None,
            path: None,
        }
    }

    pub fn new_with_json(code: u16, message: String) -> Self {
        ApiError {
            error_response: HttpResponse::new(
                Some(Body::Json(serde_json::json!({"message": message}))),
                None,
                code,
            ),
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
        ApiError::new_with_html(404, format!("IO Error: {}", error))
    }
}

impl From<Box<dyn std::error::Error>> for ApiError {
    fn from(error: Box<dyn std::error::Error>) -> ApiError {
        ApiError::new_with_json(500, error.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        ApiError::new_with_json(400, format!("JSON Serialization Error: {}", error))
    }
}

impl From<&str> for ApiError {
    fn from(error: &str) -> Self {
        ApiError::new_with_json(400, error.to_string())
    }
}

impl From<HttpParseError> for ApiError {
    fn from(error: HttpParseError) -> Self {
        ApiError::new_with_json(
            400,
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
        error_code,
        get_cannonical_reason(error_code),
        message
    );
    HttpResponse::new(
        Some(Body::Text(html)),
        Some(String::from("text/html")),
        error_code,
    )
}

fn get_cannonical_reason<'a>(status_code: u16) -> &'a str {
    match status_code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown Status Code",
    }
}
