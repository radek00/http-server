use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use scratch_server::api_error::ApiError;
use serde::{Deserialize, Serialize};

const SUFFIX: [&str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
const UNIT: f64 = 1000.0;

#[derive(Debug, Serialize, Deserialize)]
struct PathParts {
    part_name: String,
    full_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum FileType {
    Directory,
    File,
}

#[derive(Debug, Serialize, Deserialize)]
struct Files {
    path: String,
    name: String,
    file_type: FileType,
    last_modified: String,
    size: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DirectoryInfoResponse {
    paths: Vec<PathParts>,
    files: Vec<Files>,
}

fn human_bytes<T: Into<f64>>(bytes: T) -> String {
    let size = bytes.into();

    if size <= 0.0 {
        return "0 B".to_string();
    }

    let base = size.log10() / UNIT.log10();

    let result = format!("{:.1}", UNIT.powf(base - base.floor()),)
        .trim_end_matches(".0")
        .to_owned();

    [&result, SUFFIX[base.floor() as usize]].join(" ")
}

pub fn list_directory(path: &str) -> Result<serde_json::Value, ApiError> {
    let base_root_path = Path::new("./").canonicalize()?;
    let target_path = PathBuf::from("./").join(
        percent_encoding::percent_decode_str(path)
            .decode_utf8_lossy()
            .to_string(),
    );
    let cannonical_target_path = target_path.canonicalize()?;

    let current_dir = std::env::current_dir()?.canonicalize()?;
    if !cannonical_target_path.starts_with(current_dir) {
        return Err(ApiError::new_with_json(
            400,
            "Only paths relative to the current directory are allowed",
        ));
    }

    let mut current_full_path = String::new();
    let mut directory_response = DirectoryInfoResponse {
        paths: target_path
            .components()
            .map(|c| {
                let part_name = c.as_os_str().to_string_lossy().to_string();
                current_full_path.push_str(&part_name);
                current_full_path.push('/');
                PathParts {
                    part_name,
                    full_path: current_full_path.clone(),
                }
            })
            .collect(),
        files: Vec::new(),
    };

    let directory_contents = fs::read_dir(cannonical_target_path)?;

    for path in directory_contents {
        let path = path?;
        let system_time: DateTime<Utc> = path.metadata()?.modified()?.into();

        let file = Files {
            name: path.file_name().into_string().unwrap(),
            path: path
                .path()
                .strip_prefix(&base_root_path)
                .map_err(|err| ApiError::new_with_json(500, &err.to_string()))?
                .to_string_lossy()
                .into_owned(),
            file_type: if path.path().is_dir() {
                FileType::Directory
            } else {
                FileType::File
            },
            last_modified: system_time.format("%d/%m/%Y %T").to_string(),
            size: human_bytes(path.metadata()?.len() as f64),
        };
        directory_response.files.push(file);
    }

    let v = serde_json::to_value(directory_response)?;

    Ok(v)
}

pub fn parse_index_path(path: &str) -> Result<PathBuf, String> {
    let index_path = PathBuf::from(path);
    if index_path.exists() {
        Ok(index_path)
    } else {
        Err("Index file not found".to_string())
    }
}
