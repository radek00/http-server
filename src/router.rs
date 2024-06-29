use regex::Regex;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use termcolor::Color;

use crate::{logger::Logger, ApiError, Body, HttpResponse};

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
    TRACE,
    CONNECT,
}

impl HttpMethod {
    fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::TRACE => "TRACE",
            HttpMethod::CONNECT => "CONNECT",
        }
    }
}

fn get_status_code_color(status_code: u16) -> Color {
    match status_code {
        100..=199 => Color::Cyan,
        200..=299 => Color::Green,
        300..=399 => Color::Yellow,
        400..=499 => Color::Red,
        _ => Color::Magenta,
    }
}

type Handler =
    Box<dyn Fn(Option<&str>, HashMap<&str, &str>) -> Result<HttpResponse, ApiError> + Send + Sync>;

pub struct Route {
    pattern: Regex,
    handler: Handler,
    method: HttpMethod,
}
pub struct Router {
    routes: Vec<Route>,
    logger: Option<Arc<Logger>>,
    cors: Option<Cors>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            logger: None,
            cors: None,
        }
    }
    pub fn with_logger(mut self, logger: Option<Arc<Logger>>) -> Self {
        self.logger = logger;
        self
    }

    pub fn with_cors(mut self, cors: Cors) -> Self {
        self.cors = Some(cors);
        self
    }

    pub fn hsa_cors(&self) -> bool {
        self.cors.is_some()
    }

    pub fn add_route<F>(&mut self, path: &str, method: HttpMethod, handler: F)
    where
        F: Fn(Option<&str>, HashMap<&str, &str>) -> Result<HttpResponse, ApiError>
            + Send
            + Sync
            + 'static,
    {
        let pattern = if path == "/*" {
            "^.*$".to_string()
        } else {
            format!("^{}$", path.replace('{', "(?P<").replace('}', ">[^/]+)"))
        };
        let regex = Regex::new(&pattern).unwrap();
        self.routes.push(Route {
            pattern: regex,
            handler: Box::new(handler),
            method,
        });
    }

    pub fn route(
        &self,
        path: &str,
        method: &str,
        data: Option<&str>,
    ) -> Result<HttpResponse, ApiError> {
        let stripped_path: Vec<&str> = path.splitn(2, '?').collect();
        if method == HttpMethod::OPTIONS.as_str() {
            let mut response = HttpResponse::new(None, None, 204);
            if let Some(cors) = &self.cors {
                for (key, value) in &cors.headers {
                    response = response.add_response_header(key.to_string(), value.to_string());
                }
            }
            Ok(response)
        } else {
            for route in &self.routes {
                let pattern_match = route.pattern.captures(stripped_path[0]);

                match pattern_match {
                    Some(pattern_match) => {
                        if route.method.as_str() != method {
                            return Err(ApiError::new_with_json(
                                405,
                                "Method Not Allowed".to_string(),
                            ));
                        }
                        let mut param_dict: HashMap<&str, &str> = route
                            .pattern
                            .capture_names()
                            .flatten()
                            .filter_map(|n| Some((n, pattern_match.name(n)?.as_str())))
                            .collect();

                        if stripped_path.len() == 2 {
                            for param in stripped_path[1].split('&') {
                                let pair: Vec<&str> = param.split('=').collect();
                                if pair.len() == 2 {
                                    param_dict.insert(pair[0], pair[1]);
                                }
                            }
                        }
                        let mut response =
                            (route.handler)(data, param_dict).map_err(|mut err| {
                                err.method = Some(method.to_string());
                                err.path = Some(stripped_path[0].to_string());
                                err
                            })?;

                        if let Some(cors) = &self.cors {
                            for (key, value) in &cors.headers {
                                response = response
                                    .add_response_header(key.to_string(), value.to_string());
                            }
                        }

                        self.log_response(response.status_code, stripped_path[0], method)?;

                        return Ok(response);
                    }
                    None => continue,
                }
            }
            let error_response = HttpResponse::new(
                Some(Body::Json(
                    json!({"message": format!("No route found for path {}", path)}),
                )),
                None,
                404,
            );

            self.log_response(error_response.status_code, stripped_path[0], method)?;

            Ok(error_response)
        }
    }
    pub fn log_response(
        &self,
        status_code: u16,
        path: &str,
        method: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(logger) = &self.logger {
            let time_string = chrono::offset::Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            let status_code_color = get_status_code_color(status_code);

            let args = vec![
                (time_string, Some(Color::White)),
                (status_code.to_string(), Some(status_code_color)),
                (method.to_string(), Some(Color::White)),
                (path.to_string(), Some(Color::White)),
            ];

            logger.log_stdout("{} - {} - {} {}", args)?;
        }
        Ok(())
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Cors {
    headers: Vec<(String, String)>,
}

impl Cors {
    pub fn new() -> Self {
        Cors {
            headers: Vec::new(),
        }
    }

    pub fn with_origins(mut self, value: &str) -> Self {
        self.headers
            .push(("Access-Control-Allow-Origin".to_string(), value.to_string()));
        self
    }

    pub fn with_methods(mut self, value: &str) -> Self {
        self.headers.push((
            "Access-Control-Allow-Methods".to_string(),
            value.to_string(),
        ));
        self
    }

    pub fn with_headers(mut self, value: &str) -> Self {
        self.headers.push((
            "Access-Control-Allow-Headers".to_string(),
            value.to_string(),
        ));
        self
    }

    pub fn with_credentials(mut self, value: &str) -> Self {
        self.headers.push((
            "Access-Control-Allow-Credentials".to_string(),
            value.to_string(),
        ));
        self
    }
}

impl Default for Cors {
    fn default() -> Self {
        Self::new()
    }
}
