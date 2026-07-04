use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::ACCEPT_ENCODING;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::{Shutdown, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

struct TestServer {
    child: Child,
    port: u16,
}

impl TestServer {
    fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    fn https_base_url(&self) -> String {
        format!("https://127.0.0.1:{}", self.port)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn release_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("release");
    if cfg!(windows) {
        path.push("scratch-server.exe");
    } else {
        path.push("scratch-server");
    }
    assert!(
        path.exists(),
        "Release binary not found at {}. Run `cargo build --release --bin scratch-server` first.",
        path.display()
    );
    path
}

fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to ephemeral port");
    let port = listener
        .local_addr()
        .expect("Failed to resolve local addr")
        .port();
    drop(listener);
    port
}

fn spawn_server(extra_args: &[&str], capture_output: bool) -> TestServer {
    let port = find_free_port();
    let mut cmd = Command::new(release_binary_path());

    cmd.arg("--port").arg(port.to_string());
    cmd.args(extra_args);
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));

    if capture_output {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let mut child = cmd.spawn().expect("Failed to spawn release server process");

    thread::sleep(Duration::from_secs(2));

    if let Some(status) = child
        .try_wait()
        .expect("Failed to check server process status")
    {
        panic!("Server exited early with status: {status}");
    }

    TestServer { child, port }
}

fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build HTTP client")
}

fn https_client() -> Client {
    ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build HTTPS client")
}

fn testdata_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("testData");
    for part in parts {
        path.push(part);
    }
    path
}

fn fixture_text(path: &Path) -> String {
    fs::read_to_string(path).expect("Failed to read fixture file")
}

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
    let server = spawn_server(&["--cert", &cert_value, "--certpass", &cert_pass], false);

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

    // Use raw HTTP to preserve exact Authorization header casing expected by parser.
    let mut stream = TcpStream::connect(("127.0.0.1", server.port)).expect("Failed to connect");
    stream
        .write_all(
            b"GET /api/directory?path=. HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Basic dXNlcjpwYXNz\r\nConnection: close\r\n\r\n",
        )
        .expect("Failed to write request");
    let _ = stream.shutdown(Shutdown::Write);

    let mut raw_response = String::new();
    stream
        .read_to_string(&mut raw_response)
        .expect("Failed reading auth response");

    assert!(
        raw_response.starts_with("HTTP/1.1 200"),
        "Expected authorized response, got: {raw_response}"
    );
}

#[test]
fn compression_argument_sets_gzip_content_encoding() {
    let server = spawn_server(&["--ip", "127.0.0.1", "--compression"], false);

    let response = Client::builder()
        .no_gzip()
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
        Some("gzip")
    );
}

#[test]
fn index_argument_serves_custom_index_file() {
    let custom_index = testdata_path(&["public", "custom_index.html"]);
    let custom_index_value = custom_index.to_string_lossy().to_string();
    let expected = fixture_text(&custom_index);

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
