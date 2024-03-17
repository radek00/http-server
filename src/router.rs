use std::{collections::HashMap, fmt::Error};

pub struct HttpResponse {
    pub content_type: String,
    pub body: String,
    pub status_code: u16,
}

impl HttpResponse {
    pub fn new(body: String, content_type: Option<String>, status_code: u16) -> Self {
        HttpResponse {
            content_type: content_type.unwrap_or_else(|| "application/json".to_string()),
            body,
            status_code
        }
    }
}

type Handler = Box<dyn Fn(&str) -> Result<HttpResponse, Error> + Send + Sync>;

pub struct Router {
    routes: HashMap<String, Handler>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    pub fn add_route<F>(&mut self, path: &str, method: &str, handler: F)
    where
        F: Fn(&str) -> Result<HttpResponse, Error> + Send + Sync + 'static,
    {
        self.routes.insert(format!("{}{}", method, path), Box::new(handler));
    }

    pub fn route(&self, path: &str, method: &str, data: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        match self.routes.get(&format!("{}{}", method, path)) {
            Some(handler) => {
                let response = handler(data)?;
                println!("Response: {:?}", response.body);
                Ok(response)
            },
            None => {
                println!("No route found for path: {}", path);
                Ok(HttpResponse::new(
                    format!("No route found for path: {}", path),
                    None,
                    404,
                ))
            },
        }
    }
}