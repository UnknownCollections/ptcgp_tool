#![deny(missing_docs)]
//! CLI application for interacting with Pokemon TCG Pocket

/// Global memory allocator using snmalloc_rs for performance improvements.
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

pub(crate) mod binary;
pub(crate) mod commands;
pub(crate) mod crypto;
pub(crate) mod hash;
pub(crate) mod proto;
pub(crate) mod unity;
pub(crate) mod utils;
pub(crate) mod archive;

use commands::tui::run_tui;
use anyhow::Result;
use const_format::formatcp;

/// Title with auto build version.
///
/// This constant holds the application title and includes the build version
/// extracted from the environment variable `VERSION`.
pub const TITLE: &str = formatcp!("Pokemon TCG Pocket Tool - v{}", env!("VERSION"));

/// Entry point for the CLI application.
///
/// This function parses CLI arguments using `run_cli` and, if successful,
/// launches the Text-based User Interface (TUI) via `run_tui`.
fn main() -> Result<()> {
    let app_args = commands::run_cli()?;
    // If run_cli() returns, that means we aren't headless and should run the TUI
    run_tui(app_args)?;
    Ok(())
}
