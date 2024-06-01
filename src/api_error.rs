use std::{convert::Infallible, fmt};

use crate::{Body, HttpResponse};

#[derive(Debug)]
pub struct ApiError {
    pub error: HttpResponse,
    // IoError(std::io::Error),
    // JsonError(HttpResponse),
    // // Utf8Error(std::string::FromUtf8Error),
    // Other(HttpResponse), // Catch-all for any other error
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
        ApiError {
            error: format_error(500, error.to_string()),
        }
    }
}

impl From<Box<dyn std::error::Error>> for ApiError {
    fn from(error: Box<dyn std::error::Error>) -> ApiError {
        println!("Error box: {}", error);
        // Here you can define how to convert the error.
        // This is just a simple example that wraps the error message in an ApiError.
        ApiError {
            error: format_error(500, error.to_string()),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        println!("Error notmal: {}", error);
        ApiError {
            error: format_error(500, error.to_string()),
        }
    }
}

impl From<&str> for ApiError {
    fn from(error: &str) -> Self {
        // Create an ApiError from the string error message
        ApiError {
            error: format_error(500, error.to_string()),
        }
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
    let response = HttpResponse::new(
        Body::Text(html),
        Some(String::from("text/html")),
        error_code,
    );
    response
}
