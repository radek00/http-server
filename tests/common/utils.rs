use std::{net::TcpListener, path::PathBuf, process::{Child, Command, Stdio}, thread, time::Duration};

use reqwest::blocking::{Client, ClientBuilder};

pub struct TestServer {
    pub child: Child,
    pub port: u16,
}

impl TestServer {
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn https_base_url(&self) -> String {
        format!("https://127.0.0.1:{}", self.port)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn release_binary_path() -> PathBuf {
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

pub fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to ephemeral port");
    let port = listener
        .local_addr()
        .expect("Failed to resolve local addr")
        .port();
    drop(listener);
    port
}

pub fn spawn_server(extra_args: &[&str], capture_output: bool) -> TestServer {
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

pub fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(5))
        .http1_title_case_headers()
        .build()
        .expect("Failed to build HTTP client")
}

pub fn https_client() -> Client {
    ClientBuilder::new()
        .http1_title_case_headers()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build HTTPS client")
}

pub fn testdata_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/data");
    for part in parts {
        path.push(part);
    }
    path
}