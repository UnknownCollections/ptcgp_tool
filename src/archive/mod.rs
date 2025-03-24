#![allow(dead_code)]

use crate::archive::apk::ApkFile;
use crate::archive::xapk::XApkFile;
use anyhow::{bail, Result};
use std::path::Path;

mod apk;
mod xapk;

/// Trait representing a generic Android archive.
///
/// This trait abstracts operations common to Android archive formats (e.g., APK and XAPK).
/// Implementations of this trait are expected to provide functionality to read files contained within the archive.
pub trait AndroidArchive {
    /// Reads an internal file from the archive, returning its bytes.
    ///
    /// # Arguments
    ///
    /// * `internal_path` - The relative path of the file within the archive.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a vector of bytes if the file is read successfully,
    /// or an error if the file cannot be read.
    fn read_internal_file(&mut self, internal_path: &str) -> Result<Vec<u8>>;
}

/// Opens an Android archive file and returns an object that implements [`AndroidArchive`].
///
/// The archive type is determined based on the file extension provided in the path.
/// Currently, only files with `.apk` and `.xapk` extensions are supported.
///
/// # Type Parameters
///
/// * `P`: A type that can be referenced as a [`Path`].
///
/// # Arguments
///
/// * `path` - The file system path to the Android archive.
///
/// # Returns
///
/// A boxed trait object implementing [`AndroidArchive`] if the file extension is supported.
///
/// # Errors
///
/// Returns an error if:
/// - The file extension is not supported.
/// - There is an issue opening the archive using the corresponding handler.
pub fn open_archive<P: AsRef<Path>>(path: P) -> Result<Box<dyn AndroidArchive>> {
    match path.as_ref().extension().and_then(|s| s.to_str()) {
        Some("apk") => {
            let apk = ApkFile::open(path)?;
            Ok(Box::new(apk))
        }
        Some("xapk") => {
            let xapk = XApkFile::open(path)?;
            Ok(Box::new(xapk))
        }
        _ => bail!("Unsupported file extension"),
    }
}
