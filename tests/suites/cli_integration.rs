use reqwest::blocking::Client;
use reqwest::header::ACCEPT_ENCODING;
use std::fs;
use std::io::Read;
use std::time::Duration;

use crate::common::utils::{http_client, https_client, spawn_server, testdata_path};

#[test]
fn port_argument_starts_server_on_selected_port() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);
    let response = http_client()
        .get(server.base_url())
        .send()
        .expect("Request to server failed");

    assert_eq!(response.status().as_u16(), 200);
}

#[test]
fn cert_and_certpass_enable_https_requests() {
    let cert_pass = "testing";
    let cert_path = testdata_path(&["certs", "keyStore.p12"]);
    let cert_value = cert_path.to_string_lossy().to_string();
    let server = spawn_server(&["--cert", &cert_value, "--certpass", cert_pass], false);

    let response = https_client()
        .get(server.https_base_url())
        .send()
        .expect("HTTPS request failed");

    assert_eq!(response.status().as_u16(), 200);
}

#[test]
fn silent_argument_disables_startup_logging() {
    let mut noisy = spawn_server(&["--ip", "127.0.0.1"], true);
    let mut silent = spawn_server(&["--ip", "127.0.0.1", "--silent"], true);

    let _ = noisy.child.kill();
    let _ = silent.child.kill();

    let _ = noisy.child.wait();
    let _ = silent.child.wait();

    let mut noisy_stdout = String::new();
    let mut silent_stdout = String::new();

    if let Some(mut out) = noisy.child.stdout.take() {
        out.read_to_string(&mut noisy_stdout)
            .expect("Failed reading noisy stdout");
    }
    if let Some(mut out) = silent.child.stdout.take() {
        out.read_to_string(&mut silent_stdout)
            .expect("Failed reading silent stdout");
    }

    assert!(
        noisy_stdout.contains("Port:") || noisy_stdout.contains("Logs:"),
        "Expected startup output in non-silent mode, got: {noisy_stdout}"
    );
    assert!(
        !silent_stdout.contains("Port:") && !silent_stdout.contains("Logs:"),
        "Did not expect startup banner in silent mode, got: {silent_stdout}"
    );
}

#[test]
fn requests_are_logged_when_logging_enabled() {
    let mut server = spawn_server(&["--ip", "127.0.0.1"], true);
    let response = http_client()
        .get(server.base_url())
        .send()
        .expect("Request to server failed");
    assert!(response.status().is_success());

    server.child.kill().expect("Failed to kill server");
    server.child.wait().expect("Failed to wait for server");

    if let Some(mut stdout) = server.child.stdout.take() {
        let mut stdout_string = String::new();
        stdout
            .read_to_string(&mut stdout_string)
            .expect("Failed to read server stdout");
        println!("Server stdout:\n{}", stdout_string);
        assert!(stdout_string.contains("127.0.0.1"));
        assert!(stdout_string.contains("200"));
        assert!(stdout_string.contains("GET"));
        assert!(stdout_string.contains("/"));
    }
}

#[test]
fn cors_argument_adds_cors_headers_to_options_response() {
    let server = spawn_server(&["--ip", "127.0.0.1", "--cors"], false);

    let response = http_client()
        .get(server.base_url())
        .send()
        .expect("CORS request failed");

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|h| h.to_str().ok()),
        Some("*")
    );
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-methods")
            .and_then(|h| h.to_str().ok()),
        Some("GET, POST, PUT, DELETE")
    );
}

#[test]
fn ip_argument_binds_server_to_localhost() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let response = http_client()
        .get(server.base_url())
        .send()
        .expect("Localhost request failed");

    assert_eq!(response.status().as_u16(), 200);
}

#[test]
fn auth_argument_requires_credentials_and_allows_valid_basic_auth() {
    let server = spawn_server(&["--ip", "127.0.0.1", "--auth", "user:pass"], false);

    let unauthorized = http_client()
        .get(server.base_url())
        .send()
        .expect("Unauthorized request failed");

    assert_eq!(unauthorized.status().as_u16(), 401);
    assert_eq!(
        unauthorized
            .headers()
            .get("www-authenticate")
            .and_then(|h| h.to_str().ok()),
        Some("Basic")
    );

    let authorized = http_client()
        .get(server.base_url())
        .basic_auth("user", Some("pass"))
        .send()
        .expect("Authorized request failed");

    assert_eq!(authorized.status().as_u16(), 200);
}

#[test]
fn compression_argument_sets_gzip_content_encoding() {
    let server = spawn_server(&["--ip", "127.0.0.1", "--compression"], false);

    let response = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build client")
        .get(server.base_url())
        .header(ACCEPT_ENCODING, "gzip")
        .send()
        .expect("Compressed request failed");

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response
            .headers()
            .get("content-encoding")
            .and_then(|h| h.to_str().ok()),
        Some("gzip")
    );
}

#[test]
fn compression_argument_only_compresses_when_accept_encoding_is_set() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let response = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build no-gzip client")
        .get(server.base_url())
        .header(ACCEPT_ENCODING, "gzip")
        .send()
        .expect("Compressed request failed");

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response
            .headers()
            .get("content-encoding")
            .and_then(|h| h.to_str().ok()),
        None
    );
}

#[test]
fn index_argument_serves_custom_index_file() {
    let custom_index = testdata_path(&["public", "custom_index.html"]);
    let custom_index_value = custom_index.to_string_lossy().to_string();
    let expected = fs::read_to_string(&custom_index).expect("Failed to read fixture file");

    let server = spawn_server(
        &["--ip", "127.0.0.1", "--index", &custom_index_value],
        false,
    );

    let response = http_client()
        .get(format!("{}/", server.base_url()))
        .send()
        .expect("Custom index request failed");

    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().expect("Failed to read response body");
    assert_eq!(body, expected);
}
