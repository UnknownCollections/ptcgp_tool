use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use zip::read::ZipArchive;
use anyhow::Result;

/// Represents an open APK file.
///
/// This struct encapsulates a file handle for an APK, allowing access to its internal ZIP entries.
pub struct ApkFile {
    /// The file handle for the APK file on disk.
    file: File,
}

impl ApkFile {
    /// Opens an APK file from the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a type that can be converted into a `Path`.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` if the APK file is successfully opened.
    /// * `Err` if an I/O error occurs while opening the file.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self { file })
    }

    /// Reads an internal file (by its path within the APK) into a vector of bytes.
    ///
    /// This method resets the file cursor to the beginning of the APK file, constructs a ZIP archive
    /// from it, locates the specified internal file, and reads its entire contents.
    ///
    /// # Arguments
    ///
    /// * `internal_path` - The path of the file inside the APK (for example, `"AndroidManifest.xml"`).
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` containing the file’s contents if the file is found and read successfully.
    /// * `Err` if the file is not found or if an I/O error occurs during the read operation.
    pub fn read_internal_file(&mut self, internal_path: &str) -> Result<Vec<u8>> {
        // Reset the file cursor to ensure the ZIP archive is read from the beginning.
        self.file.seek(SeekFrom::Start(0))?;

        // Create a ZIP archive interface to access files within the APK.
        let mut apk_archive = ZipArchive::new(&mut self.file)?;

        // Locate the file within the archive by its name.
        let mut file_entry = apk_archive.by_name(internal_path)?;

        // Read the file's contents into a byte vector.
        let mut file_contents = Vec::new();
        file_entry.read_to_end(&mut file_contents)?;
        Ok(file_contents)
    }
}
