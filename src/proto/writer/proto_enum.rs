use crate::proto::proto_enum::{ProtoEnum, ProtoEnumVariant};
use crate::proto::writer::DEFAULT_INDENT_SIZE;
use heck::ToShoutySnakeCase;
use std::fmt::{self, Write};

impl ProtoEnum {
    /// Formats the enum in a human-readable form with the specified indentation.
    ///
    /// Enum variants are sorted by tag to ensure consistent ordering.
    ///
    /// # Parameters
    /// - `f`: A mutable reference to the string buffer where the formatted enum will be written.
    /// - `indent`: The number of spaces to indent the enum definition.
    pub fn fmt_pretty(&self, f: &mut String, indent: usize) -> fmt::Result {
        writeln!(f, "{:width$}enum {} {{", "", self.name, width = indent)?;
        let mut variants: Vec<_> = self.variants.values().collect();
        variants.sort_by_key(|v| v.tag);
        for variant in variants {
            variant.fmt_pretty(f, indent + DEFAULT_INDENT_SIZE)?;
        }
        writeln!(f, "{:width$}}}", "", width = indent)
    }

    /// Returns a pretty-formatted string representation of the enum.
    ///
    /// # Parameters
    /// - `indent`: The number of spaces to indent the enum definition.
    pub fn to_pretty_string(&self, indent: usize) -> String {
        let mut s = String::with_capacity(64);
        self.fmt_pretty(&mut s, indent).expect("Formatting error");
        s
    }
}

impl ProtoEnumVariant {
    /// Formats the enum variant in a human-readable form with the specified indentation.
    ///
    /// The variant name is converted to SHOUTY_SNAKE_CASE.
    ///
    /// # Parameters
    /// - `f`: A mutable reference to the string buffer where the formatted enum variant will be written.
    /// - `indent`: The number of spaces to indent the enum variant.
    pub fn fmt_pretty(&self, f: &mut String, indent: usize) -> fmt::Result {
        writeln!(
            f,
            "{:width$}{} = {};",
            "",
            self.name.to_shouty_snake_case(),
            self.tag,
            width = indent
        )
    }

    /// Returns a pretty-formatted string representation of the enum variant.
    ///
    /// # Parameters
    /// - `indent`: The number of spaces to indent the enum variant.
    pub fn to_pretty_string(&self, indent: usize) -> String {
        let mut s = String::with_capacity(32);
        self.fmt_pretty(&mut s, indent).expect("Formatting error");
        s
    }
}
