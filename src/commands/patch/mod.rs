use crate::binary::elf::Elf;
use crate::commands::patch::section_hashes::update_section_hash;
use crate::unity::unity_loader::load_encrypted_il2cpp;
use crate::utils::consts::{GLOBAL_METADATA_PATH, IL2CPP_PATH};
use crate::xapk::{ApkFile, XApkFile};
use anyhow::{bail, Result};
use clap::Args;
use function_hashes::update_fn_hashes;
use std::fs;
use std::path::PathBuf;

mod function_hashes;
mod section_hashes;

/// Arguments required for patching the IL2CPP file.
///
/// This struct holds the file paths for both the original and modified data sources.
/// Users can provide either an APK/XAPK archive or separate IL2CPP and global metadata files.
///
/// # Fields
/// - `xapk`: Optional path to the original XAPK file.
/// - `apk`: Optional path to the original APK file.
/// - `il2cpp`: Optional path to the original IL2CPP file.
/// - `global_metadata`: Optional path to the original global metadata file.
/// - `modified`: Path to the modified IL2CPP file that will be patched.
#[derive(Args)]
pub struct PatchArgs {
    /// Path to the original XAPK file, if available.
    #[clap(long)]
    pub xapk: Option<PathBuf>,
    /// Path to the original APK file, if available.
    #[clap(long)]
    pub apk: Option<PathBuf>,
    /// Path to the original IL2CPP file.
    #[clap(long)]
    pub il2cpp: Option<PathBuf>,
    /// Path to the original global metadata file.
    #[clap(long)]
    pub global_metadata: Option<PathBuf>,
    /// Path where the modified IL2CPP file will be written.
    pub modified: PathBuf,
}

/// Executes the IL2CPP patching process by updating function and section hashes.
///
/// This function performs the following steps:
/// 1. Reads the necessary input data (original IL2CPP, global metadata, and modified IL2CPP) using the provided arguments.
/// 2. Extracts decryption keys from the original IL2CPP data.
/// 3. Decrypts the global metadata using the extracted keys.
/// 4. Loads the IL2CPP data along with its decrypted metadata.
/// 5. Updates function hashes and section hashes in the modified IL2CPP data.
/// 6. Writes the patched IL2CPP file to the specified output path.
///
/// # Parameters
/// - `args`: The patching arguments containing file paths.
///
/// # Returns
/// A `Result` which is:
/// - `Ok(())` if patching completes successfully.
/// - An error if any step in the process fails.
pub fn execute(args: PatchArgs) -> Result<()> {
    let (il2cpp_data, global_metadata_data, modified_il2cpp_data) = get_input_data(&args)?;

    let il2cpp = load_encrypted_il2cpp(il2cpp_data, global_metadata_data)?;

    let mut modified_il2cpp = Elf::new(modified_il2cpp_data)?;

    update_fn_hashes(&il2cpp, &mut modified_il2cpp)?;

    update_section_hash(&il2cpp, &mut modified_il2cpp, vec![".text", "il2cpp"])?;

    let (_, modified_data) = modified_il2cpp.take();

    fs::write(args.modified, modified_data)?;

    Ok(())
}

/// Retrieves input data required for patching from the provided file paths.
///
/// This function reads the modified IL2CPP file and then attempts to extract the original IL2CPP and
/// global metadata data from one of the following sources:
/// - An XAPK archive (if the `xapk` field is provided).
/// - An APK archive (if the `apk` field is provided).
/// - Directly from file paths (if both `il2cpp` and `global_metadata` fields are provided).
///
/// # Parameters
/// - `args`: Reference to the `PatchArgs` containing the input file paths.
///
/// # Returns
/// A tuple `(il2cpp_data, global_metadata_data, modified_il2cpp_data)` where each element is a vector of bytes.
///
/// # Errors
/// Returns an error if:
/// - Reading any of the specified files fails.
/// - None of the expected input configurations are provided.
#[allow(clippy::type_complexity)]
fn get_input_data(args: &PatchArgs) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let modified_il2cpp = fs::read(&args.modified)?;

    if let Some(ref xapk_path) = args.xapk {
        let mut archive = XApkFile::open(xapk_path)?;
        let gm_data = archive.read_internal_file(GLOBAL_METADATA_PATH)?;
        let il2cpp_data = archive.read_internal_file(IL2CPP_PATH)?;
        Ok((il2cpp_data, gm_data, modified_il2cpp))
    } else if let Some(ref apk_path) = args.apk {
        let mut archive = ApkFile::open(apk_path)?;
        let gm_data = archive.read_internal_file(GLOBAL_METADATA_PATH)?;
        let il2cpp_data = archive.read_internal_file(IL2CPP_PATH)?;
        Ok((il2cpp_data, gm_data, modified_il2cpp))
    } else if let (Some(il2cpp_path), Some(global_metadata_path)) =
        (&args.il2cpp, &args.global_metadata)
    {
        let il2cpp_data = fs::read(il2cpp_path)?;
        let gm_data = fs::read(global_metadata_path)?;
        Ok((il2cpp_data, gm_data, modified_il2cpp))
    } else {
        bail!("Provide either --apk/--xapk or both --il2cpp and --global-metadata.")
    }
}
