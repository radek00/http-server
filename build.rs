use regex::Regex;
use std::env;
use std::ffi::OsStr;
use std::fs::{self};
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("PROFILE").unwrap() == "release" {
        let version = env::var("CARGO_PKG_VERSION")?;

        let dist_dir = Path::new("src/dist");

        let re = Regex::new(r#"src="([^"]*?)(-\d+\.\d+\.\d+)?\.js""#)?;
        for entry in fs::read_dir(dist_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.file_name().unwrap() == OsStr::new("index.html") {
                let html_content = fs::read_to_string(&path)?;
                let modified_html = re.replace_all(&html_content, |caps: &regex::Captures| {
                    format!("src=\"{}-{}.js\"", &caps[1], version)
                });

                fs::write(&path, modified_html.as_ref())?;
                continue;
            }

            let new_name = change_file_name(&path, &version);
            fs::rename(&path, &new_name)?;
        }
    }
    Ok(())
}

fn change_file_name(path: &Path, version: &str) -> PathBuf {
    let file_vec: Vec<&str> = path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .splitn(2, '-')
        .collect();
    let extension = path.extension().and_then(OsStr::to_str).unwrap_or("");
    let new_file_name = format!("{}-{}.{}", file_vec[0], version, extension);
    path.with_file_name(new_file_name)
}
