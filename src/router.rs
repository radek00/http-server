use regex::Regex;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use termcolor::Color;

use crate::{logger::Logger, Body, HttpResponse};

fn get_status_code_color(status_code: u16) -> Color {
    match status_code {
        100..=199 => Color::Cyan,
        200..=299 => Color::Green,
        300..=399 => Color::Yellow,
        400..=499 => Color::Red,
        _ => Color::Magenta,
    }
}

type Handler = Box<
    dyn Fn(Option<&str>, HashMap<&str, &str>) -> Result<HttpResponse, Box<dyn std::error::Error>>
        + Send
        + Sync,
>;

pub struct Route {
    pattern: Regex,
    handler: Handler,
}
pub struct Router {
    routes: Vec<Route>,
    logger: Option<Arc<Logger>>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            logger: None,
        }
    }
    pub fn with_logger(mut self, logger: Option<Arc<Logger>>) -> Self {
        self.logger = logger;
        self
    }

    pub fn add_route<F>(&mut self, path: &str, method: &str, handler: F)
    where
        F: Fn(
                Option<&str>,
                HashMap<&str, &str>,
            ) -> Result<HttpResponse, Box<dyn std::error::Error>>
            + Send
            + Sync
            + 'static,
    {
        let pattern = format!(
            "^{}{}$",
            method,
            path.replace('{', "(?P<").replace('}', ">[^/]+)")
        );
        let regex = Regex::new(&pattern).unwrap();
        self.routes.push(Route {
            pattern: regex,
            handler: Box::new(handler),
        });
    }

    pub fn route(
        &self,
        path: &str,
        method: &str,
        data: Option<&str>,
    ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let stripped_path: Vec<&str> = path.splitn(2, '?').collect();
        let pattern = format!("{}{}", method, stripped_path[0]);
        for route in &self.routes {
            let pattern_match = route.pattern.captures(&pattern);

            match pattern_match {
                Some(pattern_match) => {
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
                    let response = (route.handler)(data, param_dict)?;

                    self.log_response(response.status_code, stripped_path[0], method)?;

                    return Ok(response);
                }
                None => continue,
            }
        }
        let error_response = HttpResponse::new(
            Body::Json(json!({"message": format!("No route found for path {}", path)})),
            None,
            404,
        );

        self.log_response(error_response.status_code, stripped_path[0], method)?;

        Ok(error_response)
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
