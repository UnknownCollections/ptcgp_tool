use crate::archive::open_archive;
use crate::commands::proto::extractor::generate_proto_schema;
use crate::commands::tui::commands::IntoTui;
use crate::commands::tui::utils::{
    get_checkbox_value, get_optional_path, get_required_path, make_path_input, BrowseType,
};
use crate::commands::AppCommand;
use crate::proto::writer::write_entry_file;
use crate::unity::unity_loader::load_encrypted_il2cpp;
use crate::utils::consts::{
    APK_FILTER, GLOBAL_METADATA_PATH, IL2CPP_FILTER, IL2CPP_PATH, METADATA_FILTER,
};
use anyhow::{bail, Result};
use clap::Args;
use cursive::traits::{Nameable, Resizable};
use cursive::utils::markup::markdown;
use cursive::views::{Checkbox, Dialog, DummyView, LinearLayout, TextView};
use cursive::Cursive;
use log::{debug, info};
use parking_lot::Mutex;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

mod extractor;

/// Command line arguments for the extraction process.
///
/// This struct holds various optional and required file paths used during the extraction:
/// - `apk`: Optional path to an APK archive.
/// - `il2cpp`: Optional path to the il2cpp file.
/// - `global_metadata`: Optional path to the global metadata file.
/// - `output`: Required output directory where the generated protobuf files will be written.
/// - `overwrite`: Flag to allow overwriting of non-empty output directories.
#[derive(Args, Default, Clone)]
pub struct ExtractArgs {
    /// Path to an APK file.
    #[clap(long)]
    pub apk: Option<PathBuf>,
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

impl IntoTui for ExtractArgs {
    /// Converts the extraction arguments into a TUI (Text User Interface) dialog using Cursive.
    ///
    /// This method builds a dialog with two extraction methods: one for an APK/XAPK file,
    /// and another for IL2CPP and Global Metadata files. It also sets up input fields and a callback
    /// that validates and processes the user input before invoking the next command.
    ///
    /// # Arguments
    ///
    /// * `siv` - A mutable reference to the Cursive TUI instance.
    /// * `next_fn` - A callback function to be executed after the extraction arguments are processed.
    fn into_tui<F>(self, siv: &mut Cursive, next_fn: F)
    where
        F: 'static + FnOnce(&mut Cursive, AppCommand) + Send + Sync,
    {
        let cmd = Arc::new(Mutex::new(Some(self)));
        let next_fn = Arc::new(Mutex::new(Some(next_fn)));

        let dialog = Dialog::new()
            .title("Extract Arguments")
            .padding_lrtb(1, 1, 1, 0)
            .content(
                LinearLayout::vertical()
                    // Instruction text for choosing an extraction method
                    .child(TextView::new("Choose one extraction method:").center())
                    .child(DummyView.fixed_height(1))
                    // Method 1: APK/XAPK extraction
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
                    // Method 2: IL2CPP and Global Metadata extraction
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
                    // Common fields for extraction output
                    .child(make_path_input(
                        "Output Directory (*): ",
                        "output",
                        BrowseType::Folder,
                        None,
                    ))
                    .child(
                        LinearLayout::horizontal()
                            .child(TextView::new("Overwrite directory: "))
                            .child(Checkbox::new().with_checked(false).with_name("overwrite")),
                    ),
            )
            .button("Run", {
                let cmd = Arc::clone(&cmd);
                let next_fn = Arc::clone(&next_fn);
                move |s| {
                    // Lock and take our ExtractArgs
                    let mut cmd_inner = cmd.lock().take().unwrap();

                    // Fill in the arguments from the TUI input fields
                    cmd_inner.apk = get_optional_path(s, "apk");
                    cmd_inner.il2cpp = get_optional_path(s, "il2cpp");
                    cmd_inner.global_metadata = get_optional_path(s, "global_metadata");
                    cmd_inner.output = get_required_path(s, "output");
                    cmd_inner.overwrite = get_checkbox_value(s, "overwrite");

                    if let Err(err) = cmd_inner.validate() {
                        cmd.lock().replace(cmd_inner);
                        s.add_layer(
                            Dialog::text(markdown::parse(format!("**Error:**\n\n{}", err)))
                                .dismiss_button("Back"),
                        );
                        return;
                    }

                    // Pop the dialog after successful validation
                    s.pop_layer();

                    // Call the callback if present
                    if let Some(callback) = next_fn.lock().take() {
                        callback(s, AppCommand::ExtractProto(cmd_inner));
                    }
                }
            })
            .button("Cancel", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog.max_width(80));
    }

    /// Validates the extraction arguments.
    ///
    /// Checks that:
    /// - Either an APK file is provided, or both IL2CPP and Global Metadata files are given.
    /// - The provided file paths exist.
    /// - The output directory is not empty unless the overwrite flag is set, and it is a directory.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all arguments are valid.
    /// * `Err(String)` with an error message if any validation step fails.
    fn validate(&self) -> Result<(), String> {
        // Validate input file combinations.
        match (&self.apk, &self.il2cpp, &self.global_metadata) {
            // Valid: Only APK is provided.
            (Some(apk_path), None, None) => {
                if !apk_path.exists() {
                    return Err("APK file does not exist".into());
                }
            }
            // Valid: Both IL2CPP and global-metadata are provided.
            (None, Some(il2cpp_path), Some(global_path)) => {
                if !il2cpp_path.exists() {
                    return Err("IL2CPP file does not exist".into());
                }
                if !global_path.exists() {
                    return Err("Global metadata file does not exist".into());
                }
            }
            // Any other combination is invalid.
            _ => {
                return Err(
                    "Either provide an APK file or both IL2CPP and global-metadata files".into(),
                );
            }
        }

        if self.output.as_os_str().is_empty() {
            return Err("Output directory must not be empty".into());
        }

        // Validate output directory.
        if self.output.exists() {
            if !self.output.is_dir() {
                return Err("Output path is not a directory".into());
            }
            // If the directory exists and is not empty, then the overwrite flag must be set.
            match fs::read_dir(&self.output) {
                Ok(mut entries) => {
                    if entries.next().is_some() && !self.overwrite {
                        return Err(
                            "Output directory is not empty. Use --overwrite to allow overwriting."
                                .into(),
                        );
                    }
                }
                Err(e) => return Err(format!("Failed to read output directory: {}", e)),
            }
        }

        Ok(())
    }
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
    info!("Running protobuf extraction command...");
    info!(progress = 0, max = 7; "");

    info!("Loading input data...");
    let (il2cpp_data, global_metadata_data) = get_input_data(&args)?;
    info!(progress_tick = 1; "");

    info!("Decrypting global metadata and loading il2cpp...");
    let il2cpp = load_encrypted_il2cpp(il2cpp_data, global_metadata_data)?;
    info!(progress_tick = 1; "");

    info!("Generating protobuf schemas...");
    let proto_files = generate_proto_schema(il2cpp)?;
    info!(progress_tick = 1; "");

    if args.overwrite {
        info!("Overwriting output directory...");
        let _ = fs::remove_dir_all(&args.output);
        fs::create_dir_all(&args.output)?;
    }

    let mut entry_imports = Vec::new();
    info!("Writing enum definitions:");
    debug!(progress = 0, max=proto_files.count(); "");

    for en in proto_files.enums {
        info!("\t-{}", en.filename);
        let package_filepath = args.output.join(&en.filename);
        if let Some(parent) = package_filepath.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&package_filepath, en.source_code)?;
        debug!(progress_tick = 1; "");
    }
    info!(progress_tick = 1; "");

    info!("Writing message definitions:");
    for msg in proto_files.messages {
        info!("\t-{}", msg.filename);
        let package_filepath = args.output.join(&msg.filename);
        if let Some(parent) = package_filepath.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&package_filepath, msg.source_code)?;
        debug!(progress_tick = 1; "");
    }
    info!(progress_tick = 1; "");

    info!("Writing service definitions:");
    for svc in proto_files.services {
        info!("\t-{}", svc.filename);
        // Track all service definitions to make a master import file
        entry_imports.push(svc.filename.clone());
        let package_filepath = args.output.join(&svc.filename);
        if let Some(parent) = package_filepath.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&package_filepath, svc.source_code)?;
        debug!(progress_tick = 1; "");
    }
    info!(progress_tick = 1; "");

    info!("Writing master service import file...");
    write_entry_file(args.output.join("services.proto"), "pptcgp", entry_imports)?;
    info!(progress_tick = 1; "");

    info!("Done!");
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
    if let Some(ref apk_path) = args.apk {
        let mut archive = open_archive(apk_path)?;
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
