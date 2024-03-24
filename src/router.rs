use std::{collections::HashMap, fmt::Error};
use regex::Regex;
use serde_json::json;

pub enum Body {
    Text(String),
    Json(serde_json::Value),
    // Add more variants here for other types you want to support
}

pub struct HttpResponse {
    pub content_type: String,
    pub body: Body,
    pub status_code: u16,
}

impl HttpResponse {
    pub fn new(body: Body, content_type: Option<String>, status_code: u16) -> Self {
        HttpResponse {
            content_type: content_type.unwrap_or_else(|| "application/json".to_string()),
            body,
            status_code
        }
    }
}

type Handler = Box<dyn Fn(Option<&str>, Option<&HashMap<&str, &str>>) -> Result<HttpResponse, Error> + Send + Sync>;


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
        F: Fn(Option<&str>, Option<&HashMap<&str, &str>>) -> Result<HttpResponse, Error> + Send + Sync + 'static,
    {
        let pattern = format!("^{}{}$", method, path.replace("{", "(?P<").replace("}", ">[^/]+)"));
        let regex = Regex::new(&pattern).unwrap();
        self.routes.push(Route { pattern: regex, handler: Box::new(handler) });
    }

    pub fn route(&self, path: &str, method: &str, data: Option<&str>) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let stripped_path: Vec<&str> = path.split('?').collect();
        let pattern = format!("{}{}", method, stripped_path[0]);
        for route in &self.routes {
            let pattern_match = route.pattern.captures(&pattern);

            match pattern_match {
                Some(pattern_match) => {
                    let mut param_dict: HashMap<&str, &str> = route.pattern
                    .capture_names()
                    .flatten()
                    .filter_map(|n| Some((n, pattern_match.name(n)?.as_str())))
                    .collect();

                    if stripped_path[1].len() > 1 {
                        for param in stripped_path[1].split('&') {
                            let pair: Vec<&str> = param.split('=').collect();
                            if pair.len() == 2 {
                                param_dict.insert(pair[0], pair[1]);
                            }
                        }
                    }
                    let response = (route.handler)(data,  if param_dict.len() == 0 {None} else {Some(&param_dict)})?;

                    return Ok(response);
                }
                None => continue,
                
            }
        }
        println!("No route found for path: {}", path);
        Ok(HttpResponse::new(
            Body::Json(json!({"message": "No route found for path {path}"})),
            None,
            404,
        ))
    }
}