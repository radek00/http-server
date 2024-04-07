use std::collections::HashMap;

pub struct StaticFiles {
    pub content: HashMap<String, Vec<u8>>,
}

impl StaticFiles {
    pub fn new() -> Self {
        let mut content = HashMap::new();
        content.insert(String::from("script.js"), include_bytes!("./dist/script.js").to_vec());
        content.insert(String::from("index.html"), include_bytes!("./dist/index.html").to_vec());
        
        StaticFiles {
            content,
        }
    }

    pub fn get(&self, file_name: &str) -> Result<&Vec<u8>, Box<dyn std::error::Error>> {
        let file = self.content.get(file_name).ok_or("File not found")?;
        Ok(file)
    }
    
}