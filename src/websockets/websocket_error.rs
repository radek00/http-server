use std::fmt;
use std::io;

#[derive(Debug)]
pub enum WebSocketError {
    IoError(io::Error),
    HandshakeError(String),
    TextParseError(std::string::FromUtf8Error),
    UnsupportedOpCode(u8),
    FrameParseError(String),
}

impl fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebSocketError::IoError(err) => write!(f, "IO error: {}", err),
            WebSocketError::HandshakeError(err) => write!(f, "Handshake error: {}", err),
            WebSocketError::TextParseError(err) => write!(f, "UTF-8 error: {}", err),
            WebSocketError::UnsupportedOpCode(code) => write!(f, "Unsupported opcode: {}", code),
            WebSocketError::FrameParseError(err) => write!(f, "Frame parse error: {}", err),
        }
    }
}

impl std::error::Error for WebSocketError {}

impl From<io::Error> for WebSocketError {
    fn from(err: io::Error) -> WebSocketError {
        WebSocketError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for WebSocketError {
    fn from(err: std::string::FromUtf8Error) -> WebSocketError {
        WebSocketError::TextParseError(err)
    }
}
