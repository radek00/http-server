use termcolor::Color;

pub fn get_option<T>(option: &Option<T>) -> (String, Option<Color>) {
    match option {
        Some(_) => ("Enabled".to_string(), Some(Color::Green)),
        None => ("Disabled".to_string(), Some(Color::Yellow)),
    }
}
