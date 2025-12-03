use serde::{Serialize, de::DeserializeOwned};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub struct LocalStorage {
    root_dir: PathBuf,
}

impl LocalStorage {
    pub fn new() -> Self {
        let mut path = env::current_dir().expect("Failed to get current directory");
        path.push("runtime/local_storage");
        std::fs::create_dir_all(&path).expect("Failed to create local storage directory");
        Self { root_dir: path }
    }

    pub fn set<T: Serialize>(&self, key: &str, value: &T) {
        let (path, filepath) = self.get_paths(key);
        std::fs::create_dir_all(&path).expect("Failed to create local storage directory");

        let yaml = serde_yaml::to_string(value).expect("Failed to serialize value to YAML");
        let mut file = File::create(&filepath).expect("Failed to create local storage file");
        file.write_all(yaml.as_bytes())
            .expect("Failed to write value to local storage file");
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let (_path, filepath) = self.get_paths(key);
        if !filepath.exists() {
            return None;
        }

        let file = File::open(&filepath).expect("Failed to open local storage file");
        let value = serde_yaml::from_reader(file).expect("Failed to deserialize value from YAML");
        Some(value)
    }

    fn get_paths(&self, key: &str) -> (PathBuf, PathBuf) {
        let mut path = self.root_dir.clone();
        path.push(format!("{}.yaml", key));
        (self.root_dir.clone(), path)
    }
}
