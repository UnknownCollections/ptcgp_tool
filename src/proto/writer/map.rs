use crate::proto::map::ProtoMapField;
use heck::ToSnakeCase;
use std::fmt::{self, Write};

impl ProtoMapField {
    /// Formats the map field in a human-readable form with the specified indentation.
    ///
    /// # Parameters
    /// - `f`: A mutable reference to the string buffer where the formatted map field will be written.
    /// - `indent`: The number of spaces to indent the map field.
    pub fn fmt_pretty(&self, f: &mut String, indent: usize) -> fmt::Result {
        writeln!(
            f,
            "{:width$}map<{}, {}> {} = {};",
            "",
            self.key_type,
            self.value_type,
            self.name.to_snake_case(),
            self.tag,
            width = indent
        )
    }

    /// Returns a pretty-formatted string representation of the map field.
    ///
    /// # Parameters
    /// - `indent`: The number of spaces to indent the map field.
    pub fn to_pretty_string(&self, indent: usize) -> String {
        let mut s = String::with_capacity(64);
        self.fmt_pretty(&mut s, indent).expect("Formatting error");
        s
    }
}
