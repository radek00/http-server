use std::collections::HashMap;

type Handler = Box<dyn Fn(&str) -> Result<(), Box<dyn std::error::Error + Send + 'static>> + Send + Sync>;

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
        F: Fn(&str) -> Result<(), Box<dyn std::error::Error + Send + 'static>> + Send + Sync + 'static,
    {
        self.routes.insert(format!("{}{}", method, path), Box::new(handler));
    }

    pub fn route(&self, path: &str, method: &str, data: &str) -> Result<(), Box<dyn std::error::Error + Send + 'static>> {
        match self.routes.get(&format!("{}{}", method, path)) {
            Some(handler) => handler(data),
            None => {
                println!("No route found for path: {}", path);
                Ok(())
            },
        }
    }
}