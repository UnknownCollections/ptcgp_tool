use crate::commands::tui::logger::TuiProgress;
use crate::commands::tui::utils::get_checkbox_value;
use crate::commands::{AppArgs, AppCommand};
use crate::TITLE;
use anyhow::Result;
use clap::CommandFactory;
use cursive::traits::{Nameable, Resizable};
use cursive::utils::markup::markdown;
use cursive::utils::Counter;
use cursive::view::{Margins, ScrollStrategy};
use cursive::views::{
    Button, Checkbox, Dialog, DummyView, LinearLayout, ProgressBar, ScrollView, TextView,
};
use cursive::Cursive;
use cursive_aligned_view::Alignable;
use cursive_extras::VDivider;
use hashbrown::HashMap;
use log::{Level, LevelFilter};
use strum::IntoEnumIterator;

/// A trait for types that can be converted into a TUI command.
///
/// This trait provides an interface to transform a command into its TUI representation and to validate it.
pub trait IntoTui: Sized {
    /// Converts the instance into a TUI command and executes a provided callback.
    ///
    /// # Arguments
    ///
    /// * `siv` - A mutable reference to the Cursive TUI.
    /// * `next_fn` - A callback function that takes a mutable reference to Cursive and an AppCommand.
    fn into_tui<F>(self, siv: &mut Cursive, next_fn: F)
    where
        F: 'static + FnOnce(&mut Cursive, AppCommand) + Send + Sync,
        Self: Sized;

    /// Validates the instance.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the instance is valid.
    /// * `Err(String)` with an error message if invalid.
    fn validate(&self) -> Result<(), String>;
}

/// Displays the command selection screen in the TUI.
///
/// Constructs a dialog with command descriptions, selection buttons, and global options.
///
/// # Arguments
///
/// * `siv` - A mutable reference to the Cursive TUI instance.
pub fn command_select_screen(siv: &mut Cursive) {
    let args = AppArgs::command();
    // Build vertical layouts for command descriptions.
    let mut description_layout = LinearLayout::vertical();
    description_layout.add_child(TextView::new("Please select a command:\n\n"));

    // Build vertical layout for command selection buttons.
    let mut button_layout = LinearLayout::vertical();

    // Iterate over all command variants using strum's EnumIter.
    for (cmd, sub) in AppCommand::iter().zip(args.get_subcommands()) {
        let name = cmd.display_name();
        let description = sub.get_about().map(|s| s.to_string()).unwrap_or_default();

        // Add formatted description from Clap's metadata.
        description_layout.add_child(TextView::new(markdown::parse(format!("**[{name}]**"))));
        description_layout.add_child(TextView::new(description));
        description_layout.add_child(DummyView::new());

        // Add a button for the command that will trigger its TUI representation.
        button_layout.add_child(
            Button::new(name, move |s| {
                let verbose = get_checkbox_value(s, "verbose");
                cmd.clone().into_tui(s, move |s, args| {
                    let _ = command_run_view(
                        s,
                        AppArgs {
                            command: args,
                            headless: false,
                            verbose,
                        },
                    );
                });
            })
            .align_top_left(),
        );
    }

    let global_options_layout = LinearLayout::horizontal()
        .child(TextView::new("Verbose: "))
        .child(Checkbox::new().with_name("verbose"));

    // Compose the overall layout with descriptions, buttons, global options, and a quit button.
    let main_layout = LinearLayout::vertical()
        .child(description_layout.max_width(50))
        .child(button_layout.align_center())
        .child(DummyView::new())
        .child(global_options_layout.align_bottom_right())
        .child(VDivider::new())
        .child(Button::new("Quit", |s| s.quit()).align_bottom_right());

    let dialog = Dialog::around(main_layout)
        .padding(Margins::lrtb(2, 2, 1, 1))
        .title(TITLE);

    siv.add_layer(dialog);
}

/// Runs the selected command and sets up a TUI view to display progress and logs.
///
/// It initializes progress bars based on verbosity, sets up a custom logger,
/// and launches the command asynchronously while updating the UI.
///
/// # Arguments
///
/// * `siv` - A mutable reference to the Cursive TUI instance.
/// * `args` - Application arguments including command details and the verbosity flag.
///
/// # Returns
///
/// * `Ok(())` if the view is set up successfully, or an error if initialization fails.
pub fn command_run_view(siv: &mut Cursive, args: AppArgs) -> Result<()> {
    let siv_cb = siv.cb_sink().clone();

    // Set the maximum log level based on the verbosity flag.
    let level_filter = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let mut progress_bars = HashMap::new();

    // Initialize the info-level progress bar with its counter.
    let mut info_progress = ProgressBar::new();
    let info_counter = Counter::new(100);
    info_progress.set_counter(info_counter.clone());
    progress_bars.insert(Level::Info, info_counter);

    // Optionally create a debug progress bar if verbose mode is enabled.
    let debug_progress = if args.verbose {
        let mut debug_progress = ProgressBar::new();
        let debug_counter = Counter::new(100);
        debug_progress.set_counter(debug_counter.clone());
        let debug_progress = debug_progress.with_name("DEBUG_progress").full_width();
        progress_bars.insert(Level::Debug, debug_counter);
        Some(debug_progress)
    } else {
        None
    };

    // Setup the custom logger to update progress bars in the TUI.
    let logger = TuiProgress::new(progress_bars, siv_cb.clone());
    log::set_boxed_logger(Box::new(logger)).map_err(|e| anyhow::anyhow!(e))?;
    log::set_max_level(level_filter);

    // Create a scrollable text view for non-progress log messages.
    let log_view = TextView::new("").with_name("log_view");
    let scroll_log = ScrollView::new(log_view).scroll_strategy(ScrollStrategy::StickToBottom);

    // Start the info progress bar and run the command asynchronously.
    info_progress.start(move |_: Counter| {
        args.command.run().unwrap();

        // Once the command completes, display a success dialog.
        let _ = siv_cb.send(Box::new(|s: &mut Cursive| {
            s.add_layer(
                Dialog::text("Command completed successfully!")
                    .title("Done")
                    .button("Quit", |s| s.quit())
                    .button("Back", |s| {
                        s.pop_layer();
                    }),
            );
        }));
    });

    // Build the overall layout: progress bars, divider, and log view.
    let mut layout =
        LinearLayout::vertical().child(info_progress.with_name("INFO_progress").full_width());
    if let Some(debug_pb) = debug_progress {
        layout = layout.child(debug_pb);
    }
    layout = layout.child(VDivider::new());
    layout = layout.child(scroll_log);

    // Add the final fullscreen dialog layer.
    siv.add_fullscreen_layer(Dialog::around(layout).title(TITLE).full_screen());

    Ok(())
}
