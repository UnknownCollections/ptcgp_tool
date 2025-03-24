use crate::commands::tui::commands::{command_run_view, command_select_screen};
use crate::commands::tui::themes::pokemon_dark_theme;
use crate::commands::AppArgs;
use crate::TITLE;
use anyhow::Result;
use cursive::{Cursive, CursiveExt};

pub mod commands;
mod themes;
pub mod utils;
mod logger;

/// Launches the text user interface (TUI) for the application.
///
/// This function initializes the TUI context, applies a custom dark theme,
/// sets the window title, and enables automatic screen refresh. If an application
/// command is provided, it will attempt to process it (this functionality is not yet implemented).
/// Otherwise, it presents the command selection screen.
///
/// # Arguments
///
/// * `command` - An optional `AppCommand` representing a specific command to execute on startup.
///
/// # Returns
///
/// Returns a `Result<()>` which is `Ok(())` if the TUI runs successfully, or an error if something goes wrong.
pub fn run_tui(command: Option<AppArgs>) -> Result<()> {
    let mut root = Cursive::new();
    root.set_theme(pokemon_dark_theme());
    root.set_window_title(TITLE);
    root.set_autorefresh(true);

    // Process the provided command or display the command selection screen.
    if let Some(command) = command {
        command_run_view(&mut root, command)?;
    } else {
        command_select_screen(&mut root);
    }

    root.run();
    Ok(())
}
