use cursive::utils::Counter;
use cursive::views::{ProgressBar, TextView};
use cursive::{CbSink, Cursive};
use hashbrown::HashMap;
use log::kv::{Error, Key, Value, VisitSource};
use log::{Level, Metadata, Record};

/// Represents an operation to modify the progress bar.
enum ProgressOp {
    /// Sets the progress bar to a specific value.
    Set(usize),
    /// Increments the progress bar by a given amount.
    Tick(usize),
}

/// A visitor for extracting progress-related values from log key-value pairs.
struct ProgressVisitor {
    /// Optional progress operation (set or tick) parsed from the log.
    progress: Option<ProgressOp>,
    /// Optional maximum value for the progress bar.
    max: Option<usize>,
}

impl<'a> VisitSource<'a> for ProgressVisitor {
    /// Visits each key-value pair from a log record to extract progress information.
    ///
    /// Recognizes the keys "progress", "progress_tick", and "max" to update internal state.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the log entry.
    /// * `value` - The value associated with the key.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on successful visit.
    fn visit_pair(&mut self, key: Key<'a>, value: Value<'a>) -> Result<(), Error> {
        match key.as_str() {
            "progress" => {
                if let Some(n) = value.to_u64() {
                    self.progress = Some(ProgressOp::Set(n as usize));
                }
            }
            "progress_tick" => {
                if let Some(n) = value.to_u64() {
                    self.progress = Some(ProgressOp::Tick(n as usize));
                }
            }
            "max" => {
                self.max = value.to_u64().map(|n| n as usize);
            }
            _ => {}
        }
        Ok(())
    }
}

/// A logger that updates TUI progress bars and displays log messages.
///
/// It maintains a mapping of progress counters for different log levels and uses a callback sink to update the UI.
pub struct TuiProgress {
    /// Mapping of log levels to their associated progress counters.
    progress_bars: HashMap<Level, Counter>,
    /// Callback sink to send UI update events.
    siv_cb: CbSink,
}

impl TuiProgress {
    /// Creates a new instance of `TuiProgress`.
    ///
    /// # Arguments
    ///
    /// * `progress_bars` - A `HashMap` that maps each progress level to its counter.
    /// * `siv_cb` - A callback sink for sending UI update notifications.
    ///
    /// # Returns
    ///
    /// A new `TuiProgress` initialized with the provided progress bars and callback sink.
    pub fn new(progress_bars: HashMap<Level, Counter>, siv_cb: CbSink) -> Self {
        Self {
            progress_bars,
            siv_cb,
        }
    }
}

impl log::Log for TuiProgress {
    /// Determines whether a log record is enabled.
    ///
    /// This implementation enables all log records.
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    /// Processes a log record to update progress bars or append log messages.
    ///
    /// If progress-related key-value pairs are present, the corresponding progress bar is updated.
    /// Otherwise, the log message is appended to the log view.
    ///
    /// # Arguments
    ///
    /// * `record` - The log record to process.
    fn log(&self, record: &Record) {
        if record.target().starts_with("cursive") {
            return;
        }
        // Extract progress-related values from the log record.
        let mut visitor = ProgressVisitor {
            progress: None,
            max: None,
        };
        let _ = record.key_values().visit(&mut visitor);

        // Update the progress bar if a progress operation is found.
        if let Some(progress_op) = visitor.progress {
            let level = record.level();
            let max = visitor.max;
            if let Some(counter) = self.progress_bars.get(&level) {
                let _ = self.siv_cb.send(Box::new(move |s: &mut Cursive| {
                    s.call_on_name(&format!("{}_progress", level), |pb: &mut ProgressBar| {
                        if let Some(max) = max {
                            pb.set_max(max);
                        }
                    });
                }));
                match progress_op {
                    ProgressOp::Set(n) => counter.set(n),
                    ProgressOp::Tick(n) => counter.tick(n),
                }
            }
        } else {
            // Append non-progress log messages to the log view.
            let log_msg = format!("{}\n", record.args()).replace("\t", "    ");
            let _ = self.siv_cb.send(Box::new(move |s: &mut Cursive| {
                s.call_on_name("log_view", |view: &mut TextView| {
                    view.append(log_msg);
                });
            }));
        }
    }

    /// Flushes the logger.
    ///
    /// This implementation performs no operations on flush.
    fn flush(&self) {}
}
