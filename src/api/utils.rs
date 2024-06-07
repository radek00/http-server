use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
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

pub fn list_directory(path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let base_root_path = Path::new("./").canonicalize()?;
    let target_path = PathBuf::from("./").join(path);
    let cannonical_target_path = target_path.canonicalize()?;

    let current_dir = std::env::current_dir()?;
    if !cannonical_target_path.starts_with(&current_dir) {
        return Err("Only paths relative to the current directory are allowed".into());
    }

    let mut current_full_path = String::new();
    let mut directory_response = DirectoryInfoResponse {
        paths: target_path
            .components()
            .map(|c| {
                let part_name = c.as_os_str().to_string_lossy().to_string();
                current_full_path.push_str(&part_name);
                current_full_path.push('/');
                //let full_path = format!("{}/{}", current_full_path, part_name);
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
                .strip_prefix(&base_root_path)?
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
