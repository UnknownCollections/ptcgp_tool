use crate::commands::cli::run_cli_headless;
use crate::commands::tui::run_tui;
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use cursive::Cursive;
use strum::EnumIter;
use tui::commands::IntoTui;

pub mod cli;
pub mod patch;
pub mod proto;
pub mod tui;

/// Command-line arguments for the application.
///
/// This structure is automatically populated by the `clap` crate based on the input provided
/// to the executable.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct AppArgs {
    /// Specifies which subcommand to execute.
    #[clap(subcommand)]
    pub command: AppCommand,

    /// Runs the command in headless mode.
    ///
    /// In headless mode, the subcommand is executed immediately and the process exits after execution.
    #[clap(long)]
    pub headless: bool,

    /// Enables verbose logging output.
    ///
    /// When set, the application provides more detailed logging information.
    #[clap(long)]
    pub verbose: bool,
}

/// Enumerates the available subcommands for the application.
///
/// Each variant corresponds to a module that implements a specific functionality.
#[derive(Subcommand, Clone, EnumIter)]
pub enum AppCommand {
    /// Extract protobuf definitions from either an XAPK, APK, or il2cpp/metadata files.
    ExtractProto(proto::ExtractArgs),
    /// Patch the IL2CPP file to remove modification detection by updating code hashes.
    Patch(patch::PatchArgs),
}

impl AppCommand {
    /// Executes the selected subcommand.
    ///
    /// This method first validates the command parameters and then dispatches execution
    /// to the corresponding module based on the subcommand variant.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the subcommand executes successfully.
    /// * An error wrapped in `anyhow::Result` if the execution fails.
    pub fn run(&self) -> Result<()> {
        // Validate command parameters; map any validation error into an anyhow error.
        self.validate().map_err(|err| anyhow!(err))?;
        // Clone self to move into the match arms and execute the appropriate command.
        match self.clone() {
            AppCommand::ExtractProto(args) => proto::execute(args),
            AppCommand::Patch(args) => patch::execute(args),
        }
    }

    /// Returns a short, human-readable display name for the subcommand.
    ///
    /// This can be used for logging or in UI elements to identify the active command.
    ///
    /// # Returns
    ///
    /// A static string representing the display name of the command.
    pub fn display_name(&self) -> &'static str {
        match self {
            AppCommand::ExtractProto(_) => "Extract Protobuf",
            AppCommand::Patch(_) => "Patch IL2CPP",
        }
    }
}

impl IntoTui for AppCommand {
    /// Transforms the command into its Text User Interface (TUI) representation.
    ///
    /// This method integrates the command into the provided `Cursive` TUI application and
    /// sets up the next function to be executed after TUI initialization.
    ///
    /// # Parameters
    ///
    /// * `siv` - Mutable reference to the `Cursive` TUI instance.
    /// * `next_fn` - A closure to be executed after integrating the command into the TUI.
    fn into_tui<F>(self, siv: &mut Cursive, next_fn: F)
    where
        F: 'static + FnOnce(&mut Cursive, AppCommand) + Send + Sync,
        Self: Sized,
    {
        match self {
            AppCommand::ExtractProto(cmd) => cmd.into_tui(siv, next_fn),
            AppCommand::Patch(cmd) => cmd.into_tui(siv, next_fn),
        }
    }

    /// Validates the command parameters for TUI operations.
    ///
    /// This method ensures that all required parameters for TUI integration are correct
    /// before proceeding with UI initialization.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the parameters are valid.
    /// * `Err(String)` with an error message if validation fails.
    fn validate(&self) -> std::result::Result<(), String> {
        match self {
            AppCommand::ExtractProto(cmd) => cmd.validate(),
            AppCommand::Patch(cmd) => cmd.validate(),
        }
    }
}

/// Entry point for executing the application.
///
/// This function attempts to parse command-line arguments using `clap`. Based on the parsed
/// arguments, it either executes the subcommand in headless mode or launches a text user interface (TUI).
/// If no arguments are provided, the TUI is launched by default.
///
/// # Returns
///
/// * `Ok(())` if the application runs successfully.
/// * An error if the command-line parsing or command execution fails.
pub fn run() -> Result<()> {
    match AppArgs::try_parse() {
        Ok(args) => {
            if args.headless {
                run_cli_headless(args)
            } else {
                run_tui(Some(args))
            }
        }
        Err(err) => {
            // If no arguments were provided, start the TUI; otherwise, exit with the error message.
            if std::env::args().len() == 1 {
                run_tui(None)
            } else {
                err.exit();
            }
        }
    }
}
