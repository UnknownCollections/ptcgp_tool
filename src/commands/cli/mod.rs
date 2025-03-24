mod logger;

use crate::commands::cli::logger::CliLogger;
use crate::commands::AppArgs;
use crate::TITLE;
use anyhow::{anyhow, Result};
use log::{info, LevelFilter};

/// Executes the CLI application in headless mode.
///
/// This function sets up logging based on the verbosity flag provided in the
/// command line arguments, logs the application title, and then executes the
/// command specified in the arguments.
///
/// # Arguments
///
/// * `args` - An `AppArgs` struct containing command line parameters, including
///            a verbosity flag and a command to run.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the command executes successfully, or an
///                  error if any step fails.
pub fn run_cli_headless(args: AppArgs) -> Result<()> {
    // Determine the logging level:
    // Use Debug level if verbose mode is enabled, otherwise default to Info.
    let level_filter = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    // Create an instance of the CLI logger.
    let logger = CliLogger {};

    // Initialize the global logger with the CLI logger instance.
    // Converts any logger initialization error into an anyhow error.
    log::set_boxed_logger(Box::new(logger)).map_err(|e| anyhow!(e))?;
    // Apply the selected logging level.
    log::set_max_level(level_filter);

    // Log the application title using the info log level.
    info!("{TITLE}");

    // Execute the provided command and return its result.
    args.command.run()
}
