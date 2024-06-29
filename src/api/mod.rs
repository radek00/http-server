use scratch_server::{
    api_error::ApiError, Body, HttpMethod, HttpResponse, HttpServer, Router, STATIC_FILES,
};
use std::{fs::File, path::PathBuf};

use self::utils::list_directory;

mod utils;

pub fn build_server() -> HttpServer {
    let mut args = clap::Command::new("Simple HTTP Server")
            .version("1.0")
            .author("radek00")
            .about("Simlpe HTTP Server with TLS/SSL support. Implemented api endpoints allow for navigating file system directories, uploading and downloading files.")
            .arg(clap::Arg::new("port")
                .short('p')
                .value_parser(clap::value_parser!(u16))
                .default_value("7878")
                .long("port")
                .help("Sets the port number"))
            .arg(clap::Arg::new("threads")
                .short('t')
                .value_parser(clap::value_parser!(usize))
                .default_value("12")
                .long("threads")
                .help("Sets the number of threads"))
            .arg(clap::Arg::new("cert")
                .short('c')
                .value_parser(clap::value_parser!(PathBuf))
                .required(false)
                .long("cert")
                .help("TLS/SSL certificate"))
            .arg(clap::Arg::new("certpass")
                .long("certpass")
                .default_value("")
                .help("TLS/SSL certificate password"))
            .arg(clap::Arg::new("silent")
                .action(clap::ArgAction::SetTrue)
                .short('s')
                .long("silent")
                .help("Disable logging"))
            .get_matches();

    let mut server = HttpServer::build(
        args.remove_one::<u16>("port").unwrap(),
        args.remove_one::<usize>("threads").unwrap(),
        args.remove_one::<PathBuf>("cert"),
        args.remove_one::<String>("certpass"),
    );

    if !args.get_flag("silent") {
        server = server.with_logger();
    }
    server
}

pub fn create_routes() -> Box<dyn Fn(&mut Router) + Send + Sync> {
    let closure = |router: &mut Router| {
        router.add_route("/static/{file}?", HttpMethod::GET, |_, params| {
            let file_name = match params.get("file") {
                Some(file) => file,
                None => "index.html",
            };
            Ok(HttpResponse::new(
                Some(Body::StaticFile(
                    STATIC_FILES
                        .get_file(file_name)
                        .ok_or(ApiError::new_with_html(404, "File not found".to_string()))?
                        .contents(),
                    file_name.to_string(),
                )),
                Some(
                    mime_guess::from_path(file_name)
                        .first_or_text_plain()
                        .to_string(),
                ),
                200,
            )
            .add_response_header(
                "Cache-Control".to_string(),
                "public, max-age=31536000".to_string(),
            ))
        });
        router.add_route("/api/files", HttpMethod::GET, |_, params| {
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
                Some(Body::FileStream(file, file_name)),
                content_type,
                200,
            ))
        });

        router.add_route("/api/directory", HttpMethod::PUT, |_, params| {
            Ok(HttpResponse::new(
                Some(Body::Json(list_directory(
                    params.get("path").ok_or("Missing path parameter")?,
                )?)),
                None,
                200,
            ))
        });

        router.add_route("/*", HttpMethod::GET, |_, _| {
            let index = STATIC_FILES
                .get_file("index.html")
                .ok_or(ApiError::new_with_html(404, "File not found".to_string()))?
                .contents();
            Ok(HttpResponse::new(
                Some(Body::StaticFile(index, "index.html".to_string())),
                Some("text/html".to_string()),
                200,
            ))
        });
    };

    Box::new(closure)
}
