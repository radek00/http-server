pub fn has_option<T>(option: &Option<T>) -> String {
    match option {
        Some(_) => "Enabled".to_string(),
        None => "Disabled".to_string(),
    }
}
