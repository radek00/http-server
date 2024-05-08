use http_server::{Body, HttpResponse, Router, StaticFiles};
use std::fs::File;

use self::utils::{list_directory, split_path};

mod utils;

pub fn create_routes() -> Router {
    let mut router = Router::new();

    router.add_route("/{file}?", "GET", |_, params| {
        let static_files = StaticFiles::new(); // Create a new instance of StaticFiles
        let file_name = match params.get("file") {
            Some(file) => file,
            None => "index.html",
        };
        Ok(HttpResponse::new(
            Body::StaticFile(static_files.get(file_name)?, file_name.to_string()),
            Some(
                mime_guess::from_path(file_name)
                    .first_or_text_plain()
                    .to_string(),
            ),
            200,
        ))
    });
    router.add_route("/api/error", "GET", |data, _| {
        println!("Request to other path with data {}", data.unwrap());
        Ok(HttpResponse::new(
            Body::Text("Error occured".to_string()),
            Some(String::from("text/plain")),
            500,
        ))
    });
    router.add_route("/api/files", "GET", |_, params| {
        let file_path = params.get("path").ok_or("Missing path parameter")?;
        let file = File::open(file_path)?;
        Ok(HttpResponse::new(
            Body::FileStream(
                file,
                file_path.split('/').last().ok_or("Path error")?.to_string(),
            ),
            Some(
                mime_guess::from_path(file_path)
                    .first_or_octet_stream()
                    .to_string(),
            ),
            200,
        ))
    });

    router.add_route("/api/directory", "GET", |_, params| {
        //println!("Request to directory path with query param: {:?}", params.unwrap());
        Ok(HttpResponse::new(
            Body::Json(list_directory(
                params.get("path").ok_or("Missing path parameter")?,
            )?),
            None,
            200,
        ))
    });

    router.add_route("/api/path", "GET", |_, params| {
        Ok(HttpResponse::new(
            Body::Json(split_path(
                params.get("path").ok_or("Missing path parameter")?,
            )?),
            None,
            200,
        ))
    });

    router
}
