use crate::crypto::global_metadata;
use crate::unity::il2cpp::Il2Cpp;
use crate::utils::file_backed_cache::FileBackedCache;
use anyhow::{anyhow, Result};
use foldhash::fast::FixedState;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::hash::{BuildHasher, Hasher};
use log::debug;

type EncryptionKey = [u8; 16];
type EncryptionKeyXor = u64;
type EncryptionKeyParts = (EncryptionKey, EncryptionKeyXor);

/// Static cache that maps the foldhash of IL2CPP data to its corresponding
/// (metadata_key, metadata_key_xor) pair. This file-backed cache allows the
/// expensive extraction process to be reused across multiple invocations.
static KEY_CACHE: Lazy<Mutex<FileBackedCache<u64, EncryptionKeyParts>>> =
    Lazy::new(|| Mutex::new(FileBackedCache::new("il2cpp_keys")));

/// Loads and decrypts IL2CPP data using the provided global metadata.
///
/// This function performs the following steps:
/// 1. Computes a unique foldhash from the provided global metadata data.
/// 2. Attempts to retrieve the corresponding metadata key and its XOR obfuscation from a
///    file-backed cache. If not found, it extracts these keys from the IL2CPP binary.
/// 3. Caches the extracted keys for future reuse.
/// 4. Uses the keys to decrypt the global metadata.
/// 5. Loads and returns the IL2CPP binary along with its decrypted metadata.
///
/// # Arguments
///
/// * `il2cpp_data` - A vector of bytes representing the encrypted IL2CPP binary.
/// * `global_metadata_data` - A vector of bytes representing the encrypted global metadata.
///
/// # Returns
///
/// * `Result<Il2Cpp>` - On success, returns an `Il2Cpp` instance that encapsulates the loaded
///   IL2CPP binary and its decrypted metadata; on failure, returns an error indicating the issue.
pub fn load_encrypted_il2cpp<'a>(
    il2cpp_data: Vec<u8>,
    global_metadata_data: Vec<u8>,
) -> Result<Il2Cpp<'a>> {
    // Compute a unique foldhash of the global metadata to use as the cache key.
    let mut hasher = FixedState::default().build_hasher();
    hasher.write(&global_metadata_data);
    let hash_key = hasher.finish();

    // Retrieve keys from the cache using double-checked locking.
    let (metadata_key, metadata_key_xor) = {
        if let Some(&(key, key_xor)) = KEY_CACHE.lock().get(&hash_key) {
            // Found in cache; copy the values.
            (key, key_xor)
        } else {
            // Not in cache; drop the lock to extract keys without blocking others.
            let key = Il2Cpp::extract_metadata_key(&il2cpp_data)
                .ok_or_else(|| anyhow!("Could not extract global metadata encryption key"))?;
            let key_xor = Il2Cpp::extract_metadata_key_xor(&il2cpp_data)
                .ok_or_else(|| anyhow!("Could not extract global metadata key xor data"))?;

            let new_keys = (key, key_xor);

            // Re-acquire the lock and double-check if the keys were added meanwhile.
            let mut cache = KEY_CACHE.lock();
            if let Some(&(cached_key, cached_key_xor)) = cache.get(&hash_key) {
                (cached_key, cached_key_xor)
            } else {
                cache.insert(hash_key, new_keys)?;
                new_keys
            }
        }
    };

    debug!("Metadata key: {:X?}", metadata_key);
    debug!("Metadata key xor: {:X}", metadata_key_xor);

    debug!("Decrypting global metadata...");
    let decrypted_global_metadata =
        global_metadata::decrypt(&global_metadata_data, metadata_key, metadata_key_xor);

    // Load and return the IL2CPP binary along with its decrypted metadata.
    Il2Cpp::load_from_vec(il2cpp_data, decrypted_global_metadata)
}
