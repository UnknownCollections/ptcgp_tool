use crate::TITLE;
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use cursive::Cursive;
use log::info;
use std::process::exit;
use strum::EnumIter;
use tui::commands::IntoTui;

pub mod patch;
pub mod proto;
pub(crate) mod tui;

/// Command-line arguments for the application.
///
/// This structure is automatically populated by the clap crate based on the input provided
/// to the executable.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct AppArgs {
    /// Specifies which subcommand to execute.
    #[clap(subcommand)]
    pub command: AppCommand,

    /// Runs the command in headless mode. In headless mode the subcommand is executed immediately
    /// and the process exits after execution.
    #[clap(long)]
    pub headless: bool,

    /// Enables verbose logging output when set.
    #[clap(long)]
    pub verbose: bool,
}

/// Enumerates the available subcommands for the application.
///
/// Each variant corresponds to a module that implements its functionality.
#[derive(Subcommand, Clone, EnumIter)]
pub enum AppCommand {
    /// Extract protobuf definitions from either an XAPK, APK, or il2cpp/metadata files
    ExtractProto(proto::ExtractArgs),
    /// Patch the IL2CPP file to remove modification detection by updating code hashes
    Patch(patch::PatchArgs),
}

impl AppCommand {
    /// Executes the selected subcommand.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the subcommand executes successfully.
    /// * An error if the execution of the subcommand fails.
    pub fn run(&self) -> Result<()> {
        self.validate().map_err(|err| anyhow!(err))?;
        match self.clone() {
            AppCommand::ExtractProto(args) => proto::execute(args),
            AppCommand::Patch(args) => patch::execute(args),
        }
    }

    /// Returns a display name for the command.
    pub fn display_name(&self) -> &'static str {
        match self {
            AppCommand::ExtractProto(_) => "Extract Protobuf",
            AppCommand::Patch(_) => "Patch IL2CPP",
        }
    }
}

impl IntoTui for AppCommand {
    /// Transforms the command into a TUI (text user interface) representation.
    ///
    /// This method integrates the command into the provided Cursive TUI application and
    /// sets up the next function to be executed after TUI initialization.
    ///
    /// # Parameters
    ///
    /// * `siv` - Mutable reference to the Cursive TUI instance.
    /// * `next_fn` - A closure to be executed after the command is integrated into the TUI.
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
    /// # Returns
    ///
    /// * `Ok(())` if validation succeeds.
    /// * `Err(String)` containing an error message if validation fails.
    fn validate(&self) -> std::result::Result<(), String> {
        match self {
            AppCommand::ExtractProto(cmd) => cmd.validate(),
            AppCommand::Patch(cmd) => cmd.validate(),
        }
    }
}

/// Parses command-line arguments and runs the appropriate command if requested.
///
/// This function utilizes clap for argument parsing. If no arguments are provided,
/// it returns `Ok(None)` to indicate that no subcommand was invoked. If the `headless` flag
/// is set, the subcommand is executed immediately and the process terminates.
///
/// # Returns
///
/// * `Ok(Some(command))` when running in interactive (non-headless) mode, allowing further processing.
/// * `Ok(None)` if no command-line arguments were provided.
/// * An error if argument parsing fails.
pub fn run_cli() -> Result<Option<AppArgs>> {
    let args = match AppArgs::try_parse() {
        Ok(a) => a,
        Err(err) => {
            // When no arguments are provided, avoid printing an error message.
            if std::env::args().len() == 1 {
                return Ok(None);
            } else {
                err.exit();
            }
        }
    };

    if args.headless {
        info!("{TITLE}");
        args.command.run()?;
        exit(0);
    }

    Ok(Some(args))
}
