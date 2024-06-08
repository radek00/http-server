use std::fmt;

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

impl Default for HttpParseError {
    fn default() -> Self {
        HttpParseError {
            message: "An error occurred while parsing the HTTP request".to_string(),
        }
    }
}
