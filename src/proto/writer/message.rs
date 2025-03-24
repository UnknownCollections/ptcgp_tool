use crate::proto::message::ProtoMessage;
use crate::proto::writer::DEFAULT_INDENT_SIZE;
use std::fmt::{self, Write};

impl ProtoMessage {
    /// Formats the message in a human-readable form with the specified indentation.
    ///
    /// The `current_namespace` is used to determine if a field's namespace should be printed.
    /// Nested enums, messages, fields, oneof groups, and map fields are formatted recursively with increased indentation.
    ///
    /// # Parameters
    /// - `f`: A mutable reference to the string buffer where the formatted message will be written.
    /// - `indent`: The number of spaces to indent the message.
    /// - `current_namespace`: The namespace of the current context; if a field’s namespace differs, it will be included.
    pub fn fmt_pretty(
        &self,
        f: &mut String,
        indent: usize,
        current_namespace: &str,
    ) -> fmt::Result {
        writeln!(f, "{:width$}message {} {{", "", self.name, width = indent)?;

        // Format nested enums.
        for en in &self.nested_enums {
            en.fmt_pretty(f, indent + DEFAULT_INDENT_SIZE)?;
        }
        // Format nested messages.
        for msg in &self.nested_messages {
            msg.fmt_pretty(f, indent + DEFAULT_INDENT_SIZE, current_namespace)?;
        }
        // Format fields (sorted by tag) to ensure consistent ordering.
        let mut sorted_fields: Vec<_> = self.fields.iter().collect();
        sorted_fields.sort_by_key(|f| f.tag);
        for field in sorted_fields {
            let with_namespace =
                !field.namespace.is_empty() && field.namespace != current_namespace;
            field.fmt_pretty(f, indent + DEFAULT_INDENT_SIZE, with_namespace)?;
        }
        // Format oneof groups.
        for oneof in &self.oneofs {
            oneof.fmt_pretty(f, indent + DEFAULT_INDENT_SIZE, current_namespace)?;
        }
        // Format map fields (sorted by tag) to ensure consistent ordering.
        let mut sorted_map_fields: Vec<_> = self.map_fields.iter().collect();
        sorted_map_fields.sort_by_key(|m| m.tag);
        for map_field in sorted_map_fields {
            map_field.fmt_pretty(f, indent + DEFAULT_INDENT_SIZE)?;
        }
        writeln!(f, "{:width$}}}", "", width = indent)
    }

    /// Returns a pretty-formatted string representation of the message.
    ///
    /// # Parameters
    /// - `indent`: The number of spaces to indent the message.
    /// - `current_namespace`: The current namespace context for determining field namespace inclusion.
    pub fn to_pretty_string(&self, indent: usize, current_namespace: &str) -> String {
        let mut s = String::with_capacity(256);
        self.fmt_pretty(&mut s, indent, current_namespace)
            .expect("Formatting error");
        s
    }
}
