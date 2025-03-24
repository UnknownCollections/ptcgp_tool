use anyhow::Result;
use directories::ProjectDirs;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::Hash;
use std::path::PathBuf;

/// A cache that persists key-value pairs to a file in JSON format.
///
/// This structure stores cached data in memory and backs it up to disk.
/// The cache is automatically loaded from a file if it exists, or initialized as empty.
///
/// # Type Parameters
/// - `K`: The type of the keys in the cache. Must implement serialization, deserialization, equality, and hashing.
/// - `V`: The type of the values in the cache. Must implement serialization and deserialization.
pub struct FileBackedCache<K, V> {
    /// In-memory storage for key-value pairs.
    cache: HashMap<K, V>,
    /// Filesystem path to the JSON file used for persisting the cache.
    file_path: PathBuf,
}

impl<K, V> FileBackedCache<K, V>
where
    K: Serialize + for<'de> Deserialize<'de> + Eq + Hash,
    V: Serialize + for<'de> Deserialize<'de>,
{
    /// Creates a new `FileBackedCache` with the specified name.
    ///
    /// The cache will be loaded from disk if a corresponding JSON file exists;
    /// otherwise, an empty cache is initialized.
    ///
    /// # Parameters
    /// - `name`: A string slice that represents the name of the cache file (without extension).
    ///
    /// # Examples
    /// ```
    /// let mut cache = FileBackedCache::<String, String>::new("my_cache");
    /// ```
    pub fn new(name: &str) -> Self {
        // Determine the path where the cache file should be stored.
        let file_path = Self::get_file_path(name);
        // Attempt to read and deserialize the JSON file into the cache.
        // If reading or deserialization fails, initialize an empty cache.
        let cache = fs::read_to_string(&file_path)
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default();

        FileBackedCache { cache, file_path }
    }

    /// Retrieves a reference to the value associated with the specified key.
    ///
    /// # Parameters
    ///
    /// - `key`: A reference to the key whose associated value is to be returned.
    ///
    /// # Returns
    ///
    /// - `Some(&V)` if the key exists in the cache.
    /// - `None` if the key is not found.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    /// Inserts a key-value pair into the cache and saves the updated cache to disk.
    ///
    /// # Parameters
    ///
    /// - `key`: The key to insert.
    /// - `value`: The value associated with the key.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the insertion and save operation are successful.
    /// - An error if saving to disk fails.
    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        // Update the in-memory cache.
        self.cache.insert(key, value);
        // Persist the updated cache to the disk.
        self.save()?;
        Ok(())
    }

    /// Persists the current state of the cache to the associated file in JSON format.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the cache is successfully saved.
    /// - An error if serialization or file operations fail.
    pub fn save(&self) -> Result<()> {
        // Convert the in-memory cache to a JSON string.
        let serialized = serde_json::to_string(&self.cache)?;
        // Write the serialized JSON string to the file.
        fs::write(&self.file_path, serialized)?;
        Ok(())
    }

    /// Constructs the file path for the cache based on a given name.
    ///
    /// This function uses the directories crate to determine the user's configuration directory,
    /// ensures that the directory exists, and then returns a path with the provided name
    /// and a `.json` extension.
    ///
    /// # Parameters
    ///
    /// - `name`: The base name for the cache file.
    ///
    /// # Returns
    ///
    /// - A `PathBuf` representing the full path to the cache file.
    fn get_file_path(name: &str) -> PathBuf {
        // Determine the project directory using organization and application identifiers.
        let project_dirs = ProjectDirs::from("jp", "pokemon", "ptcgp_tool")
            .expect("Could not determine user configuration directory");
        let config_dir = project_dirs.config_dir();

        // Ensure the configuration directory exists; create it if necessary.
        fs::create_dir_all(config_dir).expect("Could not create configuration directory");

        // Construct the full file path with a .json extension.
        config_dir.join(format!("{name}.json"))
    }
}
