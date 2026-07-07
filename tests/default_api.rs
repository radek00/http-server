use crate::common::utils::{http_client, spawn_server, testdata_path};
use serde_json::Value;

mod common;

#[test]
fn api_directory_returns_json_with_correct_content_type() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let response = http_client()
        .get(format!(
            "{}/api/directory?path=tests/data/public",
            server.base_url()
        ))
        .send()
        .expect("Request failed");

    assert_eq!(response.status().as_u16(), 200);
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("application/json"),
        "Expected application/json, got: {content_type}"
    );
}

#[test]
fn api_directory_lists_all_expected_files() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let body: Value = http_client()
        .get(format!(
            "{}/api/directory?path=tests/data/public",
            server.base_url()
        ))
        .send()
        .expect("Request failed")
        .json()
        .expect("Failed to parse JSON");

    let files = body["files"].as_array().expect("Missing 'files' array");
    let names: Vec<&str> = files.iter().filter_map(|f| f["name"].as_str()).collect();

    for expected in &[
        "hello.txt",
        "index.html",
        "custom_index.html",
        "page.html",
        "script.js",
        "style.css",
    ] {
        assert!(
            names.contains(expected),
            "Expected '{expected}' in directory listing, got: {names:?}"
        );
    }
}

#[test]
fn api_directory_file_entry_has_required_fields() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let body: Value = http_client()
        .get(format!(
            "{}/api/directory?path=tests/data/public",
            server.base_url()
        ))
        .send()
        .expect("Request failed")
        .json()
        .expect("Failed to parse JSON");

    let files = body["files"].as_array().expect("Missing 'files' array");
    let hello = files
        .iter()
        .find(|f| f["name"] == "hello.txt")
        .expect("hello.txt not found in listing");

    assert_eq!(hello["file_type"], "File", "file_type should be 'File'");
    assert!(
        hello["path"].as_str().unwrap_or("").contains("hello.txt"),
        "path should contain the file name"
    );
    assert!(
        !hello["last_modified"].as_str().unwrap_or("").is_empty(),
        "last_modified should be non-empty"
    );
    assert!(
        !hello["size"].as_str().unwrap_or("").is_empty(),
        "size should be non-empty"
    );
}

#[test]
fn api_directory_identifies_subdirectory_entries() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let body: Value = http_client()
        .get(format!(
            "{}/api/directory?path=tests/data",
            server.base_url()
        ))
        .send()
        .expect("Request failed")
        .json()
        .expect("Failed to parse JSON");

    let files = body["files"].as_array().expect("Missing 'files' array");
    let public_entry = files
        .iter()
        .find(|f| f["name"] == "public")
        .expect("'public' subdirectory not found in listing");

    assert_eq!(
        public_entry["file_type"], "Directory",
        "Expected 'public' to be of type Directory"
    );
}

#[test]
fn api_directory_returns_breadcrumb_paths() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let body: Value = http_client()
        .get(format!(
            "{}/api/directory?path=tests/data/public",
            server.base_url()
        ))
        .send()
        .expect("Request failed")
        .json()
        .expect("Failed to parse JSON");

    let paths = body["paths"].as_array().expect("Missing 'paths' array");
    assert!(
        !paths.is_empty(),
        "Expected at least one breadcrumb in 'paths'"
    );

    let part_names: Vec<&str> = paths
        .iter()
        .filter_map(|p| p["part_name"].as_str())
        .collect();
    assert!(
        part_names.contains(&"public"),
        "Expected 'public' in breadcrumb path parts, got: {part_names:?}"
    );
}

#[test]
fn api_directory_rejects_path_outside_working_directory() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let response = http_client()
        .get(format!("{}/api/directory?path=../", server.base_url()))
        .send()
        .expect("Request failed");

    assert_eq!(
        response.status().as_u16(),
        400,
        "Expected 400 for path traversal attempt"
    );
}

#[test]
fn api_files_downloads_text_file_with_correct_body() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);
    let expected = std::fs::read_to_string(testdata_path(&["public", "hello.txt"]))
        .expect("Failed to read hello.txt fixture");

    let response = http_client()
        .get(format!(
            "{}/api/files?path=tests/data/public/hello.txt",
            server.base_url()
        ))
        .send()
        .expect("Download request failed");

    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().expect("Failed to read response body");
    assert_eq!(body, expected, "Downloaded content does not match fixture");
}

#[test]
fn api_files_sets_content_disposition_attachment_with_filename() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let response = http_client()
        .get(format!(
            "{}/api/files?path=tests/data/public/hello.txt",
            server.base_url()
        ))
        .send()
        .expect("Download request failed");

    assert_eq!(response.status().as_u16(), 200);

    let disposition = response
        .headers()
        .get("content-disposition")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    assert!(
        disposition.contains("attachment"),
        "Expected 'attachment' in Content-Disposition, got: {disposition}"
    );
    assert!(
        disposition.contains("hello.txt"),
        "Expected filename=hello.txt in Content-Disposition, got: {disposition}"
    );
}

#[test]
fn api_files_sets_text_plain_content_type_for_txt() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let response = http_client()
        .get(format!(
            "{}/api/files?path=tests/data/public/hello.txt",
            server.base_url()
        ))
        .send()
        .expect("Download request failed");

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    assert!(
        content_type.starts_with("text/plain"),
        "Expected text/plain, got: {content_type}"
    );
}

#[test]
fn api_files_sets_correct_content_types_for_web_assets() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let cases: &[(&str, &str)] = &[
        ("tests/data/public/script.js", "javascript"),
        ("tests/data/public/style.css", "css"),
        ("tests/data/public/index.html", "html"),
    ];

    for (path, expected_fragment) in cases {
        let response = http_client()
            .get(format!("{}/api/files?path={path}", server.base_url()))
            .send()
            .unwrap_or_else(|e| panic!("Request for {path} failed: {e}"));

        assert_eq!(response.status().as_u16(), 200, "Expected 200 for {path}");

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        assert!(
            content_type.contains(expected_fragment),
            "Expected content-type containing '{expected_fragment}' for {path}, got: {content_type}"
        );
    }
}

#[test]
fn api_files_returns_error_for_nonexistent_file() {
    let server = spawn_server(&["--ip", "127.0.0.1"], false);

    let response = http_client()
        .get(format!(
            "{}/api/files?path=tests/data/public/does_not_exist.txt",
            server.base_url()
        ))
        .send()
        .expect("Request failed");

    assert!(
        response.status().is_server_error() || response.status().is_client_error(),
        "Expected an error status for a missing file, got: {}",
        response.status()
    );
}
