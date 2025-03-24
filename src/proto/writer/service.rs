use crate::proto::service::{ProtoService, ProtoServiceMethod};
use crate::proto::writer::DEFAULT_INDENT_SIZE;
use heck::ToUpperCamelCase;
use std::fmt::{self, Write};

impl ProtoService {
    /// Formats the service definition in a human-readable form with the specified indentation.
    ///
    /// The service contains multiple RPC method definitions which are indented accordingly.
    ///
    /// # Parameters
    /// - `f`: A mutable reference to the string buffer where the formatted service will be written.
    /// - `indent`: The number of spaces to indent the service definition.
    pub fn fmt_pretty(&self, f: &mut String, indent: usize) -> fmt::Result {
        writeln!(f, "{:width$}service {} {{", "", self.name, width = indent)?;
        for method in &self.methods {
            method.fmt_pretty(f, indent + DEFAULT_INDENT_SIZE)?;
        }
        writeln!(f, "{:width$}}}", "", width = indent)
    }

    /// Returns a pretty-formatted string representation of the service.
    ///
    /// # Parameters
    /// - `indent`: The number of spaces to indent the service definition.
    pub fn to_pretty_string(&self, indent: usize) -> String {
        let mut s = String::with_capacity(128);
        self.fmt_pretty(&mut s, indent).expect("Formatting error");
        s
    }
}

impl ProtoServiceMethod {
    /// Formats the RPC method definition in a human-readable form with the specified indentation.
    ///
    /// This includes handling streaming for client and server if applicable.
    ///
    /// # Parameters
    /// - `f`: A mutable reference to the string buffer where the formatted RPC method will be written.
    /// - `indent`: The number of spaces to indent the RPC method definition.
    pub fn fmt_pretty(&self, f: &mut String, indent: usize) -> fmt::Result {
        write!(
            f,
            "{:width$}rpc {} (",
            "",
            self.name.to_upper_camel_case(),
            width = indent
        )?;

        if self.client_streaming {
            write!(f, "stream ")?;
        }
        write!(f, "{}) returns (", self.input_type)?;

        if self.server_streaming {
            write!(f, "stream ")?;
        }
        writeln!(f, "{});", self.output_type)
    }

    /// Returns a pretty-formatted string representation of the RPC method.
    ///
    /// # Parameters
    /// - `indent`: The number of spaces to indent the RPC method definition.
    pub fn to_pretty_string(&self, indent: usize) -> String {
        let mut s = String::with_capacity(64);
        self.fmt_pretty(&mut s, indent).expect("Formatting error");
        s
    }
}
