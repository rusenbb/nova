//! Extension storage backend.
//!
//! Provides persistent key-value storage for extensions. Each extension gets
//! its own isolated storage namespace backed by a JSON file.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

/// Extension-specific key-value storage.
///
/// Storage is backed by a JSON file in the extension's data directory.
/// Data is cached in memory and written to disk on modification.
pub struct ExtensionStorage {
    /// Extension identifier (for error messages).
    extension_id: String,
    /// Path to the storage file.
    storage_path: PathBuf,
    /// In-memory cache of stored values.
    cache: HashMap<String, Value>,
    /// Whether the cache has uncommitted changes.
    dirty: bool,
}

impl ExtensionStorage {
    /// Create a new storage instance for an extension.
    ///
    /// If the storage file exists, it will be loaded into cache.
    /// If not, an empty cache is initialized.
    pub fn new(extension_id: &str, storage_dir: PathBuf) -> Self {
        let storage_path = storage_dir.join("storage.json");

        // Try to load existing storage
        let cache = if storage_path.exists() {
            match fs::read_to_string(&storage_path) {
                Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

        Self {
            extension_id: extension_id.to_string(),
            storage_path,
            cache,
            dirty: false,
        }
    }

    /// Get a value from storage.
    pub fn get(&self, key: &str) -> Result<Option<Value>, anyhow::Error> {
        Ok(self.cache.get(key).cloned())
    }

    /// Set a value in storage.
    ///
    /// The value is immediately written to disk.
    pub fn set(&mut self, key: &str, value: Value) -> Result<(), anyhow::Error> {
        self.cache.insert(key.to_string(), value);
        self.dirty = true;
        self.flush()
    }

    /// Remove a key from storage.
    pub fn remove(&mut self, key: &str) -> Result<(), anyhow::Error> {
        if self.cache.remove(key).is_some() {
            self.dirty = true;
            self.flush()?;
        }
        Ok(())
    }

    /// Get all keys in storage.
    pub fn keys(&self) -> Result<Vec<String>, anyhow::Error> {
        Ok(self.cache.keys().cloned().collect())
    }

    /// Check if a key exists in storage.
    #[allow(dead_code)] // Will be used by extensions
    pub fn has(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Clear all storage for this extension.
    #[allow(dead_code)] // Will be used by extensions
    pub fn clear(&mut self) -> Result<(), anyhow::Error> {
        self.cache.clear();
        self.dirty = true;
        self.flush()
    }

    /// Flush cached changes to disk.
    pub fn flush(&mut self) -> Result<(), anyhow::Error> {
        if !self.dirty {
            return Ok(());
        }

        // Ensure parent directory exists
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create storage directory for extension '{}': {}",
                    self.extension_id,
                    e
                )
            })?;
        }

        // Write storage file
        let contents = serde_json::to_string_pretty(&self.cache)?;
        fs::write(&self.storage_path, contents).map_err(|e| {
            anyhow::anyhow!(
                "Failed to write storage for extension '{}': {}",
                self.extension_id,
                e
            )
        })?;

        self.dirty = false;
        Ok(())
    }
}

impl Drop for ExtensionStorage {
    fn drop(&mut self) {
        // Best-effort flush on drop
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = ExtensionStorage::new("test-ext", temp_dir.path().to_path_buf());

        // Set and get
        storage
            .set("key1", serde_json::json!("value1"))
            .unwrap();
        assert_eq!(
            storage.get("key1").unwrap(),
            Some(serde_json::json!("value1"))
        );

        // Get non-existent key
        assert_eq!(storage.get("nonexistent").unwrap(), None);

        // Set complex value
        storage
            .set("complex", serde_json::json!({"nested": {"array": [1, 2, 3]}}))
            .unwrap();
        assert_eq!(
            storage.get("complex").unwrap(),
            Some(serde_json::json!({"nested": {"array": [1, 2, 3]}}))
        );
    }

    #[test]
    fn test_storage_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path().to_path_buf();

        // Create storage and set values
        {
            let mut storage = ExtensionStorage::new("test-ext", storage_dir.clone());
            storage.set("persistent", serde_json::json!(42)).unwrap();
        }

        // Create new storage instance and verify persistence
        {
            let storage = ExtensionStorage::new("test-ext", storage_dir);
            assert_eq!(storage.get("persistent").unwrap(), Some(serde_json::json!(42)));
        }
    }

    #[test]
    fn test_storage_remove() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = ExtensionStorage::new("test-ext", temp_dir.path().to_path_buf());

        storage.set("to_remove", serde_json::json!("value")).unwrap();
        assert!(storage.has("to_remove"));

        storage.remove("to_remove").unwrap();
        assert!(!storage.has("to_remove"));
        assert_eq!(storage.get("to_remove").unwrap(), None);
    }

    #[test]
    fn test_storage_keys() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = ExtensionStorage::new("test-ext", temp_dir.path().to_path_buf());

        storage.set("a", serde_json::json!(1)).unwrap();
        storage.set("b", serde_json::json!(2)).unwrap();
        storage.set("c", serde_json::json!(3)).unwrap();

        let mut keys = storage.keys().unwrap();
        keys.sort();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_storage_clear() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = ExtensionStorage::new("test-ext", temp_dir.path().to_path_buf());

        storage.set("a", serde_json::json!(1)).unwrap();
        storage.set("b", serde_json::json!(2)).unwrap();

        storage.clear().unwrap();
        assert!(storage.keys().unwrap().is_empty());
    }
}
