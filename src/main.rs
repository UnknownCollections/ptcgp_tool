#![deny(missing_docs)]
//! CLI application for interacting with Pokemon TCG Pocket

/// Global memory allocator using snmalloc_rs for performance improvements.
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

pub(crate) mod archive;
pub(crate) mod binary;
pub(crate) mod commands;
pub(crate) mod crypto;
pub(crate) mod hash;
pub(crate) mod proto;
pub(crate) mod unity;
pub(crate) mod utils;

use crate::unity::generated::SUPPORTED_VERSION_NAME;
use anyhow::Result;
use const_format::formatcp;

/// Title with auto build version.
///
/// This constant holds the application title and includes the build version
/// extracted from the environment variable `VERSION`.
pub const TITLE: &str = formatcp!(
    "Pokemon TCG Pocket Tool - v{} ({})",
    env!("VERSION"),
    SUPPORTED_VERSION_NAME
);

/// Main entry point for the CLI application.
///
/// This function initializes the command processing by calling the command runner,
/// and propagates any errors encountered during execution using `anyhow::Result`.
fn main() -> Result<()> {
    commands::run()
}
