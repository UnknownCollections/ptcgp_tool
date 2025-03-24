use crate::archive::open_archive;
use crate::binary::elf::Elf;
use crate::commands::patch::section_hashes::update_section_hash;
use crate::commands::tui::commands::IntoTui;
use crate::commands::tui::utils::{
    get_optional_path, get_required_path, make_path_input, BrowseType,
};
use crate::commands::AppCommand;
use crate::unity::unity_loader::load_encrypted_il2cpp;
use crate::utils::consts::{
    APK_FILTER, GLOBAL_METADATA_PATH, IL2CPP_FILTER, IL2CPP_PATH, METADATA_FILTER,
};
use anyhow::{bail, Result};
use clap::Args;
use cursive::traits::Resizable;
use cursive::utils::markup::markdown;
use cursive::views::{Dialog, DummyView, LinearLayout, TextView};
use cursive::Cursive;
use function_hashes::update_fn_hashes;
use log::info;
use parking_lot::Mutex;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

mod function_hashes;
mod section_hashes;

/// Arguments required for patching the IL2CPP file.
///
/// This struct holds the file paths for both the original and modified data sources.
/// Users can supply input in one of two ways:
/// 1. A single APK/XAPK archive that contains the IL2CPP and global metadata files.
/// 2. Separate IL2CPP and global metadata file paths.
///
/// # Fields
/// - `apk`: Optional path to the original APK/XAPK file.
/// - `il2cpp`: Optional path to the original IL2CPP file.
/// - `global_metadata`: Optional path to the original global metadata file.
/// - `modified`: Path to the modified IL2CPP file that will be patched.
#[derive(Args, Default, Clone)]
pub struct PatchArgs {
    /// Optional path to the original APK/XAPK file.
    #[clap(long)]
    pub apk: Option<PathBuf>,
    /// Optional path to the original IL2CPP file.
    #[clap(long)]
    pub il2cpp: Option<PathBuf>,
    /// Optional path to the original global metadata file.
    #[clap(long)]
    pub global_metadata: Option<PathBuf>,
    /// Path where the modified IL2CPP file will be written.
    pub modified: PathBuf,
}

impl IntoTui for PatchArgs {
    /// Converts `PatchArgs` into a text-based user interface (TUI) form.
    ///
    /// This method creates a dialog window to let users choose an input method:
    /// either providing an APK/XAPK file, or providing separate IL2CPP and Global Metadata files.
    /// It also includes an input for the required modified IL2CPP file.
    /// When the user clicks "Run", the TUI fields are read and used to update `PatchArgs`,
    /// and the provided callback is executed.
    ///
    /// # Parameters
    /// - `siv`: A mutable reference to the Cursive TUI instance.
    /// - `next_fn`: A callback function to execute after successfully gathering input.
    fn into_tui<F>(self, siv: &mut Cursive, next_fn: F)
    where
        F: 'static + FnOnce(&mut Cursive, AppCommand) + Send + Sync,
    {
        let cmd = Arc::new(Mutex::new(Some(self)));
        let next_fn = Arc::new(Mutex::new(Some(next_fn)));

        let dialog = Dialog::new()
            .title("Patch Arguments")
            .padding_lrtb(1, 1, 1, 0)
            .content(
                LinearLayout::vertical()
                    // Explain available input methods
                    .child(TextView::new("Choose one input method:").center())
                    .child(DummyView.fixed_height(1))
                    // Method 1: Using an APK/XAPK file.
                    .child(TextView::new(markdown::parse(
                        "**Method 1: Provide an APK/XAPK file:**",
                    )))
                    .child(make_path_input(
                        "(X)APK File: ",
                        "apk",
                        BrowseType::File,
                        Some(APK_FILTER),
                    ))
                    .child(DummyView.fixed_height(1))
                    // Method 2: Using separate IL2CPP and Global Metadata files.
                    .child(TextView::new(markdown::parse(
                        "**Method 2: Provide IL2CPP and Global Metadata files:**",
                    )))
                    .child(make_path_input(
                        "IL2CPP File: ",
                        "il2cpp",
                        BrowseType::File,
                        Some(IL2CPP_FILTER),
                    ))
                    .child(make_path_input(
                        "Global Metadata File: ",
                        "global_metadata",
                        BrowseType::File,
                        Some(METADATA_FILTER),
                    ))
                    .child(DummyView.fixed_height(1))
                    // Required modified IL2CPP file input.
                    .child(make_path_input(
                        "Modified IL2CPP File (*): ",
                        "modified",
                        BrowseType::File,
                        Some(IL2CPP_FILTER),
                    )),
            )
            .button("Run", {
                let cmd = Arc::clone(&cmd);
                let next_fn = Arc::clone(&next_fn);
                move |s| {
                    // Update command arguments with paths gathered from the TUI.
                    let mut cmd = cmd.lock().take().unwrap();
                    cmd.apk = get_optional_path(s, "apk");
                    cmd.il2cpp = get_optional_path(s, "il2cpp");
                    cmd.global_metadata = get_optional_path(s, "global_metadata");
                    cmd.modified = get_required_path(s, "modified");

                    // Close the dialog window.
                    s.pop_layer();

                    // Invoke the callback with the patched command arguments.
                    if let Some(callback) = next_fn.lock().take() {
                        callback(s, AppCommand::Patch(cmd));
                    }
                }
            })
            .button("Cancel", |s| {
                s.pop_layer();
            });

        // Add the dialog to the TUI with a set maximum width.
        siv.add_layer(dialog.max_width(80));
    }

    /// Validates that the provided file paths in `PatchArgs` exist and are combined correctly.
    ///
    /// It checks:
    /// - The modified IL2CPP file exists.
    /// - The input is either an APK (with no IL2CPP or metadata provided) or both IL2CPP and Global Metadata files are provided.
    /// - That any provided APK, IL2CPP, or Global Metadata file exists.
    ///
    /// # Returns
    /// - `Ok(())` if all validations pass.
    /// - An `Err(String)` with an error message if any check fails.
    fn validate(&self) -> Result<(), String> {
        // Ensure the modified file exists.
        if !self.modified.exists() {
            return Err("Modified IL2CPP File doesn't exist".into());
        }

        // Determine which optional files have been provided.
        let apk_provided = self.apk.is_some();
        let il2cpp_provided = self.il2cpp.is_some();
        let global_provided = self.global_metadata.is_some();

        // Accept only one valid combination:
        // Either an APK is provided (and no IL2CPP or Global Metadata),
        // or both IL2CPP and Global Metadata are provided (and no APK).
        match (apk_provided, il2cpp_provided, global_provided) {
            (true, false, false) | (false, true, true) => { /* valid combo */ }
            _ => {
                return Err(
                    "Only an (X)APK or both IL2CPP and Global metadata files can be provided"
                        .into(),
                );
            }
        }

        // Verify the existence of the APK file if provided.
        if let Some(apk) = &self.apk {
            if !apk.exists() {
                return Err("(X)APK File doesn't exist".into());
            }
        }

        // Verify the existence of the IL2CPP and Global Metadata files if provided.
        if let (Some(il2cpp), Some(global_metadata)) = (&self.il2cpp, &self.global_metadata) {
            if !global_metadata.exists() {
                return Err("Global metadata file doesn't exist".into());
            }
            if !il2cpp.exists() {
                return Err("Il2CPP file doesn't exist".into());
            }
        }

        Ok(())
    }
}

/// Executes the IL2CPP patching process by updating function and section hashes.
///
/// The patching process involves:
/// 1. Loading input data (original IL2CPP, global metadata, and modified IL2CPP).
/// 2. Decrypting global metadata using keys from the original IL2CPP.
/// 3. Loading and preparing both the original and modified IL2CPP data.
/// 4. Updating function hashes and section hashes in the modified IL2CPP.
/// 5. Writing the patched IL2CPP file to the specified output path.
///
/// # Parameters
/// - `args`: The patching arguments containing file paths for input and output.
///
/// # Returns
/// - `Ok(())` if the patching process completes successfully.
/// - An error if any of the steps fail.
pub fn execute(args: PatchArgs) -> Result<()> {
    info!("Running il2cpp patch command...");
    info!(progress = 0, max = 6; "");

    info!("Loading input data...");
    let (il2cpp_data, global_metadata_data, modified_il2cpp_data) = get_input_data(&args)?;
    info!(progress_tick = 1; "");

    info!("Decrypting global metadata and loading il2cpp...");
    let il2cpp = load_encrypted_il2cpp(il2cpp_data, global_metadata_data)?;
    info!(progress_tick = 1; "");

    info!("Loading modified il2cpp...");
    let mut modified_il2cpp = Elf::new(modified_il2cpp_data)?;
    info!(progress_tick = 1; "");

    info!("Updating function hashes...");
    update_fn_hashes(&il2cpp, &mut modified_il2cpp)?;
    info!(progress_tick = 1; "");

    info!("Updating section hash...");
    update_section_hash(&il2cpp, &mut modified_il2cpp, vec![".text", "il2cpp"])?;
    info!(progress_tick = 1; "");

    info!("Writing patched il2cpp file...");
    let (_, modified_data) = modified_il2cpp.take();
    fs::write(args.modified, modified_data)?;
    info!(progress_tick = 1; "");

    Ok(())
}

/// Retrieves the input data required for patching based on the provided file paths.
///
/// Depending on the provided arguments, this function extracts data using one of the following methods:
/// - If an APK is provided, it opens the archive and reads the internal IL2CPP and Global Metadata files.
/// - If separate IL2CPP and Global Metadata files are provided, it reads them directly.
///
/// The modified IL2CPP file is always read from the provided path.
///
/// # Parameters
/// - `args`: A reference to `PatchArgs` containing input file paths.
///
/// # Returns
/// A tuple `(il2cpp_data, global_metadata_data, modified_il2cpp_data)` where each element is a vector of bytes.
///
/// # Errors
/// Returns an error if reading any file fails or if the provided arguments do not match the expected configurations.
#[allow(clippy::type_complexity)]
fn get_input_data(args: &PatchArgs) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    // Read the modified IL2CPP file data.
    let modified_il2cpp = fs::read(&args.modified)?;

    // If an APK path is provided, extract IL2CPP and metadata from the archive.
    if let Some(ref apk_path) = args.apk {
        let mut archive = open_archive(apk_path)?;
        let gm_data = archive.read_internal_file(GLOBAL_METADATA_PATH)?;
        let il2cpp_data = archive.read_internal_file(IL2CPP_PATH)?;
        Ok((il2cpp_data, gm_data, modified_il2cpp))
    } else if let (Some(il2cpp_path), Some(global_metadata_path)) =
        (&args.il2cpp, &args.global_metadata)
    {
        // Otherwise, read the IL2CPP and Global Metadata files directly.
        let il2cpp_data = fs::read(il2cpp_path)?;
        let gm_data = fs::read(global_metadata_path)?;
        Ok((il2cpp_data, gm_data, modified_il2cpp))
    } else {
        bail!("Provide either --apk/--xapk or both --il2cpp and --global-metadata.")
    }
}
