use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
    
    pub async fn save_file(&self, content: &[u8], subpath: &str, filename: &str) -> Result<PathBuf> {
        let dir_path = self.base_path.join(subpath);
        fs::create_dir_all(&dir_path).await?;
        
        let file_path = dir_path.join(filename);
        fs::write(&file_path, content).await?;
        
        Ok(file_path)
    }
    
    pub async fn read_file(&self, subpath: &str, filename: &str) -> Result<Vec<u8>> {
        let file_path = self.base_path.join(subpath).join(filename);
        let content = fs::read(file_path).await?;
        
        Ok(content)
    }
    
    pub async fn delete_file(&self, subpath: &str, filename: &str) -> Result<()> {
        let file_path = self.base_path.join(subpath).join(filename);
        fs::remove_file(file_path).await?;
        
        Ok(())
    }
    
    pub async fn list_files(&self, subpath: &str) -> Result<Vec<String>> {
        let dir_path = self.base_path.join(subpath);
        if !dir_path.exists() {
            return Ok(Vec::new());
        }
        
        let mut entries = fs::read_dir(dir_path).await?;
        let mut filenames = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_file() {
                    if let Some(filename) = entry.file_name().to_str() {
                        filenames.push(filename.to_string());
                    }
                }
            }
        }
        
        Ok(filenames)
    }
    
    pub fn generate_unique_filename(original_filename: &str) -> String {
        let extension = Path::new(original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let uuid = Uuid::new_v4();
        
        if extension.is_empty() {
            format!("{}", uuid)
        } else {
            format!("{}.{}", uuid, extension)
        }
    }
}