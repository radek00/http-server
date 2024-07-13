use clap::builder::{
    styling::{AnsiColor, Effects},
    Styles,
};
use scratch_server::{
    api_error::ApiError, Body, Cors, HttpMethod, HttpResponse, HttpServer, Router, STATIC_FILES,
};
use std::{fs::File, path::PathBuf};

use self::utils::list_directory;

mod utils;

pub fn build_server() -> (HttpServer, bool) {
    let username_password_validator = |s: &str| {
        if s.contains(':') && s.split(':').count() == 2 {
            Ok(s.to_owned())
        } else {
            Err(String::from("The format must be username:password"))
        }
    };
    let mut auth = false;
    let mut args = clap::Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author("radek00")
            .styles(    Styles::styled()
            .header(AnsiColor::BrightGreen.on_default() | Effects::BOLD)
            .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
            .placeholder(AnsiColor::Yellow.on_default()))
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
                .hide_default_value(true)
                .help("TLS/SSL certificate password"))
            .arg(clap::Arg::new("silent")
                .action(clap::ArgAction::SetTrue)
                .short('s')
                .long("silent")
                .help("Disable logging"))
            .arg(clap::Arg::new("cors")
                .long("cors")
                .action(clap::ArgAction::SetTrue)
                .help("Enable CORS with Access-Control-Allow-Origin header set to *"))
            .arg(clap::Arg::new("ip")
                .long("ip")
                .default_value("0.0.0.0")
                .value_parser(clap::value_parser!(std::net::IpAddr))
                .help("Ip address to bind to"))
            .arg(clap::Arg::new("auth")
                .long("auth")
                .short('a')
                .value_parser(username_password_validator)
                .help("Enable HTTP Basic Auth. Pass username:password as argument"))
            .arg(clap::Arg::new("compression")
                .long("compression")
                .action(clap::ArgAction::SetTrue)
                .help("Enable gzip response compression"))
            .get_matches();

    let mut server = HttpServer::build(
        args.remove_one::<u16>("port").unwrap(),
        args.remove_one::<usize>("threads").unwrap(),
        args.remove_one::<PathBuf>("cert"),
        args.remove_one::<String>("certpass"),
        args.remove_one::<std::net::IpAddr>("ip").unwrap(),
        args.remove_one::<bool>("compression").unwrap(),
    );

    if let Some(credentials) = args.remove_one::<String>("auth") {
        let credentials = credentials.split(':').collect::<Vec<&str>>();
        server = server.with_credentials(credentials[0], credentials[1]);
        auth = true;
    }

    if !args.get_flag("silent") {
        server = server.with_logger();
    }

    if args.get_flag("cors") {
        server = server.with_cors_policy(
            Cors::new()
                .with_origins("*")
                .with_methods("GET, POST, PUT, DELETE")
                .with_headers("Content-Type, Authorization")
                .with_credentials("true"),
        );
    }
    (server, auth)
}

pub fn create_routes(authorize: bool) -> Box<dyn Fn(&mut Router) + Send + Sync> {
    let closure = move |router: &mut Router| {
        router.add_route(
            "/static/{file}?",
            HttpMethod::GET,
            |_, params| {
                let file_name = match params.get("file") {
                    Some(file) => file,
                    None => "index.html",
                };
                Ok(HttpResponse::new(
                    Some(Body::StaticFile(
                        STATIC_FILES
                            .get_file(file_name)
                            .ok_or(ApiError::new_with_html(404, "File not found"))?
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
                .add_response_header("Cache-Control", "public, max-age=31536000"))
            },
            authorize,
        );
        router.add_route(
            "/api/files",
            HttpMethod::GET,
            |_, params| {
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
            },
            authorize,
        );

        router.add_route(
            "/api/directory",
            HttpMethod::GET,
            |_, params| {
                Ok(HttpResponse::new(
                    Some(Body::Json(list_directory(
                        params.get("path").ok_or("Missing path parameter")?,
                    )?)),
                    None,
                    200,
                ))
            },
            authorize,
        );

        router.add_route(
            "/*",
            HttpMethod::GET,
            |_, _| {
                let index = STATIC_FILES
                    .get_file("index.html")
                    .ok_or(ApiError::new_with_html(404, "File not found"))?
                    .contents();
                Ok(HttpResponse::new(
                    Some(Body::StaticFile(index, "index.html".to_string())),
                    Some("text/html".to_string()),
                    200,
                ))
            },
            authorize,
        );
    };

    Box::new(closure)
}
