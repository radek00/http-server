use std::{collections::HashMap, fmt::Error};
use regex::Regex;

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

type Handler = Box<dyn Fn(Option<&str>, Option<&str>) -> Result<HttpResponse, Error> + Send + Sync>;


pub struct Route {
    pattern: Regex,
    handler: Handler,
}
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
        }
    }

    pub fn add_route<F>(&mut self, path: &str, method: &str, handler: F)
    where
        F: Fn(Option<&str>, Option<&str>) -> Result<HttpResponse, Error> + Send + Sync + 'static,
    {
        //self.routes.insert(format!("{}{}", method, path), Box::new(handler));

        let pattern = format!("^{}{}$", method, path.replace("{", "(?P<").replace("}", ">[^/]+)"));
        let regex = Regex::new(&pattern).unwrap();
        self.routes.push(Route { pattern: regex, handler: Box::new(handler) });
    }

    pub fn route(&self, path: &str, method: &str, data: Option<&str>) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let pattern = format!("{}{}", method, path);
        for route in &self.routes {
            let pattern_match = route.pattern.find(&pattern);

            match pattern_match {
                Some(pattern_match) => {
                    let &param = pattern_match.as_str().split('/').collect::<Vec<&str>>().last().unwrap(); 
                    let response = (route.handler)(data,  if param.is_empty() {None} else {Some(param)})?;

                    return Ok(response);
                }
                None => continue,
                
            }
        }
        println!("No route found for path: {}", path);
        Ok(HttpResponse::new(
            format!("No route found for path: {}", path),
            None,
            404,
        ))
    }
}