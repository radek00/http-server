use std::{collections::HashMap, io::BufReader};

use crate::{api_error::ApiError, ReadWrite};

#[cfg(feature = "websockets")]

pub struct WebSocket<'a> {
    reader: &'a mut BufReader<&'a mut dyn ReadWrite>,
}

impl<'a> WebSocket<'a> {
    pub fn new(reader: &'a mut BufReader<&'a mut dyn ReadWrite>) -> WebSocket<'a> {
        WebSocket { reader }
    }

    pub fn connect(&self, headers: &HashMap<&str, &str>) -> Result<(), ApiError> {
        println!("Connecting to WebSocket server...");
        if let Some(key) = headers.get("Sec-WebSocket-Key") {
            println!("WebSocket key: {}", key);
        }
        //must return upgrade response. Either do it here or only return a struct and write response in HttpResponse.
        Ok(())
    }
}
