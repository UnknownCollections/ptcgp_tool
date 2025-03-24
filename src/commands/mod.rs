use anyhow::Result;
use clap::{Parser, Subcommand};

mod patch;
mod proto;

/// Represents the command-line arguments for the application.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The subcommand to execute.
    #[clap(subcommand)]
    pub command: Command,
}

/// Enumerates the subcommands available to the application.
#[derive(Subcommand)]
pub enum Command {
    /// Extract protobuf definitions.
    ExtractProto(proto::ExtractArgs),
    /// Patch an il2cpp hash.
    Patch(patch::PatchArgs),
}

/// Executes the application by parsing the command-line arguments and dispatching the appropriate subcommand.
///
/// # Returns
///
/// * `Ok(())` if the execution was successful.
/// * An error of type `Box<dyn std::error::Error>` if any subcommand execution fails.
pub fn run() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::ExtractProto(extract_args) => proto::execute(extract_args),
        Command::Patch(patch_args) => patch::execute(patch_args),
    }
}
