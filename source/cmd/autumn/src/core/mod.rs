pub mod prelude {}

pub mod internal {
    pub use super::prelude::*;
}

use serde::{Serialize, de::DeserializeOwned};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn get_paths(key: &str) -> (PathBuf, PathBuf) {
    let mut path = env::current_dir().expect("Failed to get current directory");
    path.push("runtime/local_storage");

    let mut filepath = path.clone();
    filepath.push(format!("{}.yaml", key));
    (path, filepath)
}

pub fn local_storage_set<T: Serialize>(key: &str, value: &T) {
    let (path, filepath) = get_paths(key);
    std::fs::create_dir_all(&path).expect("Failed to create local storage directory");

    let yaml = serde_yaml::to_string(value).expect("Failed to serialize value to YAML");
    let mut file = File::create(&filepath).expect("Failed to create local storage file");
    file.write_all(yaml.as_bytes())
        .expect("Failed to write value to local storage file");
}

pub fn local_storage_get<T: DeserializeOwned>(key: &str) -> Option<T> {
    let (path, filepath) = get_paths(key);
    if !filepath.exists() {
        return None;
    }

    let file = File::open(&filepath).expect("Failed to open local storage file");
    let value = serde_yaml::from_reader(file).expect("Failed to deserialize value from YAML");
    Some(value)
}
