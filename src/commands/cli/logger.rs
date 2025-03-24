use log::kv::{Error, Key, Value, VisitSource};
use log::{Metadata, Record};

/// Visitor that inspects log record key-value pairs to detect progress-related keys.
///
/// This visitor checks for keys such as "progress", "progress_tick", or "max" in the key-value
/// pairs of a log record. When these keys are found, the corresponding boolean fields are set.
struct ProgressVisitor {
    /// Set to `true` if a "progress" or "progress_tick" key is encountered.
    progress: bool,
    /// Set to `true` if a "max" key is encountered.
    max: bool,
}

impl<'a> VisitSource<'a> for ProgressVisitor {
    /// Inspects a single key-value pair from a log record.
    ///
    /// This function sets the `progress` or `max` fields to `true` if the key matches
    /// "progress", "progress_tick", or "max" respectively.
    ///
    /// # Arguments
    ///
    /// * `key` - The key from the log record.
    /// * `_` - The associated value (unused in this implementation).
    ///
    /// # Returns
    ///
    /// * `Ok(())` indicating successful processing of the pair.
    fn visit_pair(&mut self, key: Key<'a>, _: Value<'a>) -> Result<(), Error> {
        match key.as_str() {
            "progress" => self.progress = true,
            "progress_tick" => self.progress = true,
            "max" => self.max = true,
            _ => {}
        }
        Ok(())
    }
}

/// A simple command-line logger that implements the `log::Log` trait.
///
/// This logger prints formatted log messages to the console, ignoring any log records that
/// contain progress-related key-value pairs.
pub struct CliLogger {}

impl log::Log for CliLogger {
    /// Determines if a log message with the provided metadata should be logged.
    ///
    /// This implementation always returns `true`, meaning all messages are considered enabled.
    ///
    /// # Arguments
    ///
    /// * `_metadata` - The metadata associated with the log message.
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    /// Processes and outputs a log record.
    ///
    /// The function creates a `ProgressVisitor` to inspect the log record's key-value pairs.
    /// If a progress-related key is found, the log message is skipped. Otherwise, it formats the
    /// message by replacing tab characters with four spaces and prints it to the console.
    ///
    /// # Arguments
    ///
    /// * `record` - The log record containing the message and associated key-value pairs.
    fn log(&self, record: &Record) {
        let mut visitor = ProgressVisitor {
            progress: false,
            max: false,
        };
        let _ = record.key_values().visit(&mut visitor);

        if visitor.progress || visitor.max {
            return;
        }

        let log_msg = format!("{}", record.args()).replace("\t", "    ");
        println!("{}", log_msg);
    }

    /// Flushes any buffered log records.
    ///
    /// This logger writes directly to the console, so this method does not perform any operations.
    fn flush(&self) {}
}
