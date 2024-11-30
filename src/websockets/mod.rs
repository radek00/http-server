use std::{collections::HashMap, io::BufReader};

use base64::{prelude::BASE64_STANDARD, Engine};
use sha1::{Digest, Sha1};

use crate::{api_error::ApiError, ReadWrite};

#[cfg(feature = "websockets")]

pub struct WebSocket<'a> {
    reader: &'a mut BufReader<&'a mut dyn ReadWrite>,
}

impl<'a> WebSocket<'a> {
    pub fn new(reader: &'a mut BufReader<&'a mut dyn ReadWrite>) -> WebSocket<'a> {
        WebSocket { reader }
    }

    pub fn connect(&mut self, headers: &HashMap<&str, &str>) -> Result<(), ApiError> {
        println!("Connecting to WebSocket server...");
        if let Some(key) = headers.get("Sec-WebSocket-Key") {
            println!("WebSocket key: {}", key);
            self.perform_handshake(key)?;
            loop {}
        }
        //must return upgrade response. Either do it here or only return a struct and write response in HttpResponse.
        Ok(())
    }

    fn perform_handshake(&mut self, key: &str) -> Result<(), ApiError> {
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
        let result = hasher.finalize();
        let encoded = BASE64_STANDARD.encode(result);
        println!("Encoded key: {}", encoded);
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
            Upgrade: websocket\r\n\
            Connection: Upgrade\r\n\
            Sec-WebSocket-Accept: {}\r\n\r\n",
            encoded
        );
        self.reader.get_mut().write_all(response.as_bytes())?;
        Ok(())
    }
}
