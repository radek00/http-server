use std::collections::HashMap;

pub struct StaticFiles {
    pub content: HashMap<String, &'static [u8]>,
}

impl StaticFiles {
    pub fn new() -> Self {
        let mut content: HashMap<String, &'static [u8]> = HashMap::new();
        content.insert(
            String::from("script.js"),
            include_bytes!("./dist/script.js"),
        );
        content.insert(
            String::from("index.html"),
            include_bytes!("./dist/index.html"),
        );

        StaticFiles { content }
    }

    pub fn get(&self, file_name: &str) -> Result<&'static [u8], Box<dyn std::error::Error>> {
        let file = self.content.get(file_name).ok_or("File not found")?;
        Ok(file)
    }
}

impl Default for StaticFiles {
    fn default() -> Self {
        Self::new()
    }
}
