use crate::proto::field::{ProtoCardinality, ProtoField};
use crate::proto::writer::format_package_name;
use heck::ToSnakeCase;
use std::fmt::{self, Write};

impl ProtoField {
    /// Formats the field in a human-readable form with the specified indentation.
    ///
    /// If `with_namespace` is `true`, the field type is prefixed with its namespace.
    ///
    /// # Parameters
    /// - `f`: A mutable reference to the string buffer where the formatted field will be written.
    /// - `indent`: The number of spaces to indent the field.
    /// - `with_namespace`: Whether to include the field's namespace in the output.
    pub fn fmt_pretty(&self, f: &mut String, indent: usize, with_namespace: bool) -> fmt::Result {
        write!(f, "{:width$}", "", width = indent)?;
        let field_type_str = if with_namespace {
            format!(
                "{}.{}",
                format_package_name(&self.namespace),
                self.field_type
            )
        } else {
            self.field_type.clone()
        };
        match self.cardinality {
            ProtoCardinality::Single => writeln!(
                f,
                "{} {} = {};",
                field_type_str,
                self.name.to_snake_case(),
                self.tag
            ),
            _ => writeln!(
                f,
                "{} {} {} = {};",
                self.cardinality,
                field_type_str,
                self.name.to_snake_case(),
                self.tag
            ),
        }
    }

    /// Returns a pretty-formatted string representation of the field.
    ///
    /// # Parameters
    /// - `indent`: The number of spaces to indent the field.
    /// - `with_namespace`: Whether to include the field's namespace in the output.
    pub fn to_pretty_string(&self, indent: usize, with_namespace: bool) -> String {
        let mut s = String::with_capacity(64);
        self.fmt_pretty(&mut s, indent, with_namespace)
            .expect("Formatting error");
        s
    }
}

impl fmt::Display for ProtoCardinality {
    /// Formats the protobuf field cardinality as a string for display purposes.
    ///
    /// Returns "optional" for optional fields, "repeated" for repeated fields, and an empty string for single fields.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtoCardinality::Optional => write!(f, "optional"),
            ProtoCardinality::Repeated => write!(f, "repeated"),
            ProtoCardinality::Single => Ok(()),
        }
    }
}
