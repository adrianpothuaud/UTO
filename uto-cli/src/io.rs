//! I/O utilities for JSON and files.

use std::fs;
use std::path::Path;

pub fn read_json<T: for<'de> serde::Deserialize<'de>>(path: impl AsRef<Path>) -> Result<T, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn write_json<T: serde::Serialize>(path: impl AsRef<Path>, value: &T) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}
