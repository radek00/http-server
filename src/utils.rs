const SUFFIX: [&str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
const UNIT: f64 = 1000.0;

pub fn human_bytes<T: Into<f64>>(bytes: T) -> String {
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