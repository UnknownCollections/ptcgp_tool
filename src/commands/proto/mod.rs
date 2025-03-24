use crate::commands::proto::extractor::generate_proto_schema;
use crate::proto::writer::write_entry_file;
use crate::unity::unity_loader::load_encrypted_il2cpp;
use crate::utils::consts::{GLOBAL_METADATA_PATH, IL2CPP_PATH};
use crate::xapk::{ApkFile, XApkFile};
use anyhow::{bail, Result};
use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};

mod extractor;

/// Command line arguments for the extraction process.
///
/// This struct holds various optional and required file paths used during the extraction:
/// - `apk`: Optional path to an APK archive.
/// - `xapk`: Optional path to an XAPK archive.
/// - `il2cpp`: Optional path to the il2cpp file.
/// - `global_metadata`: Optional path to the global metadata file.
/// - `output`: Required output directory where the generated protobuf files will be written.
/// - `overwrite`: Flag to allow overwriting of non-empty output directories.
#[derive(Args)]
pub struct ExtractArgs {
    /// Path to an APK file.
    #[clap(long)]
    pub apk: Option<PathBuf>,
    /// Path to an XAPK file.
    #[clap(long)]
    pub xapk: Option<PathBuf>,
    /// Path to the il2cpp file.
    #[clap(long)]
    pub il2cpp: Option<PathBuf>,
    /// Path to the global-metadata file.
    #[clap(long)]
    pub global_metadata: Option<PathBuf>,
    /// Output directory for protobuf files.
    #[clap(long)]
    pub output: PathBuf,
    /// Overwrite output directory if not empty.
    #[clap(long)]
    pub overwrite: bool,
}

/// Executes the extraction process.
///
/// This function performs the following steps:
/// 1. Validates the output directory based on the provided path and overwrite flag.
/// 2. Loads input data (il2cpp and global metadata) from either an XAPK/APK archive or individual files.
/// 3. Extracts decryption keys from the il2cpp data.
/// 4. Decrypts the global metadata using the extracted keys.
/// 5. Generates protobuf schemas from the decrypted global metadata and il2cpp data.
/// 6. Writes the generated protobuf files (enums, messages, and services) to the output directory,
///    creating subdirectories as necessary, and writes an entry file referencing the service files.
///
/// # Errors
///
/// Returns an error if any step fails, such as:
/// - Invalid output directory conditions.
/// - Missing or unreadable input files.
/// - Failure during decryption or schema generation.
/// - File I/O errors during writing.
pub fn execute(args: ExtractArgs) -> Result<()> {
    validate_output_directory(&args.output, args.overwrite)?;

    // Load the input data from the specified sources.
    let (il2cpp_data, global_metadata_data) = get_input_data(&args)?;

    let il2cpp = load_encrypted_il2cpp(il2cpp_data, global_metadata_data)?;

    let proto_files = generate_proto_schema(il2cpp)?;

    // Remove any pre-existing content.
    let _ = fs::remove_dir_all(&args.output);
    fs::create_dir_all(&args.output)?;

    let mut entry_imports = Vec::new();
    // Write enum definitions.
    for en in proto_files.enums {
        let package_filepath = args.output.join(&en.filename);
        if let Some(parent) = package_filepath.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&package_filepath, en.source_code)?;
    }
    // Write message definitions.
    for msg in proto_files.messages {
        let package_filepath = args.output.join(&msg.filename);
        if let Some(parent) = package_filepath.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&package_filepath, msg.source_code)?;
    }
    // Write service definitions and collect filenames for the entry file.
    for svc in proto_files.services {
        entry_imports.push(svc.filename.clone());
        let package_filepath = args.output.join(&svc.filename);
        if let Some(parent) = package_filepath.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&package_filepath, svc.source_code)?;
    }
    // Generate an entry file that imports all service files.
    write_entry_file(args.output.join("services.proto"), "pptcgp", entry_imports)?;

    Ok(())
}

/// Validates that the output directory is suitable for writing the protobuf files.
///
/// This function ensures:
/// - The output path exists and is a directory.
/// - The directory is empty, unless the `overwrite` flag is set.
///
/// # Errors
///
/// Returns an error if:
/// - The output path exists but is not a directory.
/// - The directory is not empty and `overwrite` is false.
fn validate_output_directory(output: &Path, overwrite: bool) -> Result<()> {
    if output.exists() {
        if !output.is_dir() {
            bail!("Output path '{}' is not a directory.", output.display());
        }
        if output.read_dir()?.next().is_some() && !overwrite {
            bail!(
                "Output directory '{}' is not empty. Use --overwrite to replace.",
                output.display()
            );
        }
    }
    Ok(())
}

/// Retrieves il2cpp and global metadata data from the provided input sources.
///
/// Depending on the provided arguments, this function will attempt to:
/// - Open an XAPK archive and extract files at predefined paths.
/// - Open an APK archive and extract files at predefined paths.
/// - Read individual il2cpp and global metadata files from the file system.
///
/// # Errors
///
/// Returns an error if:
/// - Neither archive nor individual file paths are provided.
/// - Reading from the archive or file system fails.
fn get_input_data(args: &ExtractArgs) -> Result<(Vec<u8>, Vec<u8>)> {
    if let Some(ref xapk_path) = args.xapk {
        let mut archive = XApkFile::open(xapk_path)?;
        let gm_data = archive.read_internal_file(GLOBAL_METADATA_PATH)?;
        let il2cpp_data = archive.read_internal_file(IL2CPP_PATH)?;
        Ok((il2cpp_data, gm_data))
    } else if let Some(ref apk_path) = args.apk {
        let mut archive = ApkFile::open(apk_path)?;
        let gm_data = archive.read_internal_file(GLOBAL_METADATA_PATH)?;
        let il2cpp_data = archive.read_internal_file(IL2CPP_PATH)?;
        Ok((il2cpp_data, gm_data))
    } else if let (Some(il2cpp_path), Some(global_metadata_path)) =
        (&args.il2cpp, &args.global_metadata)
    {
        let il2cpp_data = fs::read(il2cpp_path)?;
        let gm_data = fs::read(global_metadata_path)?;
        Ok((il2cpp_data, gm_data))
    } else {
        bail!("Please provide either --apk/--xapk or both --il2cpp and --global-metadata.")
    }
}
