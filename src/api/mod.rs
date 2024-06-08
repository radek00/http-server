use scratch_server::{api_error::ApiError, Body, HttpResponse, Router, StaticFiles};
use std::{fs::File, path::PathBuf};

use self::utils::list_directory;

mod utils;

pub fn create_routes(router: &mut Router) {
    router.add_route("/{file}?", "GET", |_, params| {
        let static_files = StaticFiles::new(); // Create a new instance of StaticFiles
        let file_name = match params.get("file") {
            Some(file) => file,
            None => "index.html",
        };
        Ok(HttpResponse::new(
            Body::StaticFile(
                static_files
                    .get(file_name)
                    .map_err(|err| ApiError::new_with_html(404, err.to_string()))?,
                file_name.to_string(),
            ),
            Some(
                mime_guess::from_path(file_name)
                    .first_or_text_plain()
                    .to_string(),
            ),
            200,
        ))
    });
    router.add_route("/api/files", "GET", |_, params| {
        let file_path = PathBuf::from(params.get("path").ok_or("Missing path parameter")?);
        let file_name = file_path
            .file_name()
            .ok_or("No file name")?
            .to_string_lossy()
            .to_string();
        let content_type = Some(
            mime_guess::from_path(&file_name)
                .first_or_octet_stream()
                .to_string(),
        );
        let file = File::open(file_path)?;
        Ok(HttpResponse::new(
            Body::FileStream(file, file_name),
            content_type,
            200,
        ))
    });

    router.add_route("/api/directory", "GET", |_, params| {
        Ok(HttpResponse::new(
            Body::Json(list_directory(
                params.get("path").ok_or("Missing path parameter")?,
            )?),
            None,
            200,
        ))
    });
}
