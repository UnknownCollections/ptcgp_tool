use crate::proto::circular::ProtoMessageGroup;
use std::fmt::{self};

impl ProtoMessageGroup {
    /// Formats the group of messages in a human-readable form with the specified indentation.
    ///
    /// Each message in the group is formatted and separated by a newline.
    ///
    /// # Parameters
    /// - `f`: A mutable reference to the string buffer where the formatted messages will be written.
    /// - `indent`: The number of spaces to indent each message.
    /// - `current_namespace`: The namespace of the current context for proper namespace handling.
    pub fn fmt_pretty(
        &self,
        f: &mut String,
        indent: usize,
        current_namespace: &str,
    ) -> fmt::Result {
        for msg in self.iter() {
            msg.fmt_pretty(f, indent, current_namespace)?;
            f.push('\n');
        }

        Ok(())
    }

    /// Returns a pretty-formatted string representation of the message group.
    ///
    /// # Parameters
    /// - `indent`: The number of spaces to indent each message.
    /// - `current_namespace`: The namespace of the current context.
    pub fn to_pretty_string(&self, indent: usize, current_namespace: &str) -> String {
        let mut s = String::with_capacity(256);
        self.fmt_pretty(&mut s, indent, current_namespace)
            .expect("Formatting error");
        s
    }
}
