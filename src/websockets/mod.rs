use std::{collections::HashMap, io::BufReader};

use base64::{prelude::BASE64_STANDARD, Engine};
use sha1::{Digest, Sha1};
use websocket_error::WebSocketError;

use crate::ReadWrite;

pub mod websocket_error;

#[cfg(feature = "websockets")]
#[derive(Debug)]
pub enum Frame {
    Text(Vec<u8>),
    Binary(Vec<u8>),
    Ping,
    Pong,
    Close,
}

#[cfg(feature = "websockets")]
pub struct WebSocket<'a> {
    reader: &'a mut BufReader<&'a mut dyn ReadWrite>,
}

impl<'a> WebSocket<'a> {
    pub fn new(reader: &'a mut BufReader<&'a mut dyn ReadWrite>) -> WebSocket<'a> {
        WebSocket { reader }
    }

    pub fn connect(&mut self, headers: &HashMap<&str, &str>) -> Result<(), WebSocketError> {
        println!("Connecting to WebSocket server...");
        if let Some(key) = headers.get("Sec-WebSocket-Key") {
            println!("WebSocket key: {}", key);
            self.perform_handshake(key)?;
            self.handle_connection()?;
            return Ok(());
        }
        Err(WebSocketError::HandshakeError(
            "No Sec-WebSocket-Key header".to_string(),
        ))
    }

    fn perform_handshake(&mut self, key: &str) -> Result<(), WebSocketError> {
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

    fn send_ping(&mut self) -> Result<usize, WebSocketError> {
        println!("Ping sent");
        Ok(self.reader.get_mut().write(&[0x89, 0x00])?)
    }

    fn send_pong(&mut self) -> Result<usize, WebSocketError> {
        println!("Pong sent");
        Ok(self.reader.get_mut().write(&[0x8A, 0x00])?)
    }

    fn send_text(&mut self, data: Vec<u8>) -> Result<usize, WebSocketError> {
        let text_data = String::from_utf8(data).unwrap();
        println!("Text frame: {}", text_data);
        let mut text_frame = Vec::new();
        text_frame.push(0x81);

        let length = text_data.len();

        if length < 126 {
            text_frame.push(length as u8);
        } else if length < 65536 {
            text_frame.push(126);
            text_frame.extend_from_slice(&length.to_be_bytes());
        } else {
            text_frame.push(127);
            text_frame.extend_from_slice(&length.to_be_bytes());
        }

        text_frame.extend_from_slice(text_data.as_bytes());
        Ok(self.reader.get_mut().write(&text_frame)?)
    }

    fn handle_connection(&mut self) -> Result<(), WebSocketError> {
        let mut buffer = [0; 2048];
        let mut pong_received = false;
        loop {
            if pong_received {
                self.send_ping()?;
                pong_received = false;
            }
            let buffer_size = self.reader.get_mut().read(&mut buffer)?;
            if buffer_size > 0 {
                match self.parse_frame(&buffer) {
                    Ok(frame) => match frame {
                        Frame::Text(data) => {
                            self.send_text(data)?;
                        }
                        Frame::Binary(data) => {
                            println!("Binary frame: {:?}", data);
                        }
                        Frame::Ping => {
                            self.send_pong()?;
                        }
                        Frame::Pong => {
                            pong_received = true;
                        }
                        Frame::Close => {
                            println!("Close frame received");
                            break;
                        }
                    },
                    Err(_) => todo!(),
                }
            }
        }
        Ok(())
    }
    fn parse_frame(&mut self, buffer: &[u8]) -> Result<Frame, WebSocketError> {
        if buffer.len() < 2 {
            return Err(WebSocketError::FrameParseError(
                "Frame too short".to_string(),
            ));
        }

        let first_byte = buffer[0];

        let opcode = first_byte & 0x0F; // Determines opcode

        let second_byte = buffer[1];
        let masked = (second_byte & 0x80) != 0;

        let mut payload_len = (second_byte & 0x7F) as usize;

        if !masked {
            return Err(WebSocketError::FrameParseError(
                "Frame not masked".to_string(),
            ));
        }

        let mut offset = 2;

        if payload_len == 126 {
            if buffer.len() < 4 {
                return Err(WebSocketError::FrameParseError(
                    "Frame too short".to_string(),
                ));
            }

            payload_len = u16::from_be_bytes([buffer[offset], buffer[offset + 1]]) as usize;
            offset += 2;
        } else if payload_len == 127 {
            todo!("Message fragmentation not supported");
        }

        if buffer.len() < offset + 4 + payload_len {
            return Err(WebSocketError::FrameParseError(
                "Frame too short".to_string(),
            ));
        }

        let mask = &buffer[offset..offset + 4];

        offset += 4;

        let mut data = Vec::with_capacity(payload_len);
        for i in 0..payload_len {
            data.push(buffer[offset + i] ^ mask[i % 4]);
        }

        Ok(match opcode {
            0x01 => Frame::Text(data),
            0x02 => Frame::Binary(data),
            0x08 => Frame::Close,
            0x09 => Frame::Ping,
            0x0A => Frame::Pong,
            _ => return Err(WebSocketError::UnsupportedOpCode(opcode)),
        })
    }
}
