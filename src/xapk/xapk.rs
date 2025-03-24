use anyhow::{anyhow, Result};
use hashbrown::HashMap;
use serde::Deserialize;
use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use zip::read::ZipArchive;

/// Represents the contents of a manifest.json file that describes an XAPK package.
///
/// # Fields
/// - **package_name**: The unique identifier for the application (e.g., "com.example.app").
/// - **name**: The human-readable name of the application.
/// - **version_code**: The internal version code, typically an integer represented as a string.
/// - **version_name**: The user-visible version name.
/// - **split_apks**: A list of split APK entries included in the XAPK.
#[derive(Debug, Deserialize)]
pub struct Manifest {
    /// The unique identifier for the application.
    pub package_name: String,
    /// The human-readable name of the application.
    pub name: String,
    /// The internal version code, typically represented as a string.
    pub version_code: String,
    /// The user-visible version name.
    pub version_name: String,
    /// List of split APK entries contained in the manifest.
    pub split_apks: Vec<SplitApk>,
}

/// Represents a single split APK entry as defined in the manifest.
///
/// # Fields
/// - **file**: The filename of the split APK within the XAPK archive.
/// - **id**: A unique identifier for the split APK.
#[derive(Debug, Deserialize)]
pub struct SplitApk {
    /// The filename of the split APK within the XAPK.
    pub file: String,
    /// A unique identifier for the split APK.
    pub id: String,
}

/// Encapsulates an open XAPK file and provides methods for accessing its internal files.
///
/// The `XApkFile` structure holds:
/// - an open file handle for the XAPK,
/// - an index mapping internal file paths (found within the APKs inside) to the split APK filename that contains them.
pub struct XApkFile {
    /// The open XAPK file handle.
    file: File,
    /// Index mapping an internal file path to the split APK filename that provides it.
    file_map: HashMap<String, String>,
}

impl XApkFile {
    /// Opens an XAPK file, parses its manifest, and builds an index that maps internal file paths to the
    /// corresponding split APK filename.
    ///
    /// If multiple split APKs contain the same file, entries from later split APKs in the manifest override earlier ones.
    ///
    /// # Arguments
    /// - `path`: A reference to the path of the XAPK file to open.
    ///
    /// # Errors
    /// Returns an `io::Error` if the file cannot be opened, the manifest cannot be read, or any IO or parsing error occurs.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Open the XAPK file.
        let mut file = File::open(path)?;

        // Use an inner block to limit the lifetime of mutable borrows from `file`.
        let file_map = {
            // Create a ZipArchive for reading the XAPK file's contents.
            let mut xapk_archive = ZipArchive::new(&mut file)?;

            // Retrieve and read the manifest.json file from the archive.
            let mut manifest_file = xapk_archive.by_name("manifest.json")?;
            let mut manifest_contents = String::new();
            manifest_file.read_to_string(&mut manifest_contents)?;
            // End the mutable borrow for the manifest file.
            drop(manifest_file);

            // Deserialize the manifest contents into a Manifest struct.
            let manifest: Manifest = serde_json::from_str(&manifest_contents)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            // Build an index mapping internal file paths to the split APK filename that contains them.
            let mut file_map = HashMap::new();
            for split_apk in &manifest.split_apks {
                // Attempt to locate the split APK file within the XAPK archive.
                let mut apk_entry = match xapk_archive.by_name(&split_apk.file) {
                    Ok(entry) => entry,
                    Err(_) => continue, // Skip if the split APK file is missing.
                };

                // Read the entire contents of the split APK file into memory.
                let mut apk_bytes = Vec::new();
                apk_entry.read_to_end(&mut apk_bytes)?;
                // End the mutable borrow for the split APK entry.
                drop(apk_entry);

                // Open the split APK as a ZIP archive using an in-memory cursor.
                let cursor = Cursor::new(apk_bytes);
                let mut apk_archive = ZipArchive::new(cursor)?;
                // Iterate over all files within the split APK.
                for i in 0..apk_archive.len() {
                    let file_entry = apk_archive.by_index(i)?;
                    let internal_path = file_entry.name().to_string();
                    // Map the internal file path to the split APK filename. Later entries override earlier ones.
                    file_map.insert(internal_path, split_apk.file.clone());
                }
            }

            // Return the completed file mapping.
            file_map
        }; // End inner block; all mutable borrows from xapk_archive have ended.

        // Reset the file cursor to the beginning for subsequent operations.
        file.seek(SeekFrom::Start(0))?;

        Ok(Self { file, file_map })
    }

    /// Reads an internal file from one of the split APKs contained within the XAPK.
    ///
    /// This function looks up which split APK contains the desired internal file using the pre-built file map,
    /// then extracts and returns the file's contents as a vector of bytes.
    ///
    /// # Arguments
    /// - `internal_path`: The path of the internal file to be read (as it appears within a split APK).
    ///
    /// # Errors
    /// Returns an `io::Error` if the internal file or its corresponding split APK cannot be found,
    /// or if any IO or ZIP processing error occurs during the read operation.
    pub fn read_internal_file(&mut self, internal_path: &str) -> Result<Vec<u8>> {
        // Look up which split APK file contains the requested internal file.
        let split_apk_filename = self
            .file_map
            .get(internal_path)
            .ok_or(anyhow!("File not found"))?;

        // Reset the XAPK file's cursor to the beginning.
        self.file.seek(SeekFrom::Start(0))?;
        let mut xapk_archive = ZipArchive::new(&mut self.file)?;

        // Locate and read the split APK file entry from the XAPK archive.
        let mut split_apk_entry = xapk_archive.by_name(split_apk_filename)?;
        let mut split_apk_bytes = Vec::new();
        split_apk_entry.read_to_end(&mut split_apk_bytes)?;
        // End the mutable borrow for the split APK entry.
        drop(split_apk_entry);

        // Open the split APK as a ZIP archive from the in-memory buffer.
        let cursor = Cursor::new(split_apk_bytes);
        let mut apk_archive = ZipArchive::new(cursor)?;

        // Locate and read the requested internal file from the split APK.
        let mut file_entry = apk_archive.by_name(internal_path)?;
        let mut file_contents = Vec::new();
        file_entry.read_to_end(&mut file_contents)?;
        Ok(file_contents)
    }
}
