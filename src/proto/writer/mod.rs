
mod field;
mod map;
mod message;
mod message_group;
mod one_of;
mod proto_enum;
mod service;

use hashbrown::HashSet;
use heck::ToSnakeCase;
use itertools::Itertools;
use std::fmt::{self, Write};
use std::fs::File;
use std::io;
use std::path::PathBuf;

/// The default indentation size (number of spaces) used for formatting nested structures.
pub(crate) const DEFAULT_INDENT_SIZE: usize = 2;

/// Represents a generated protobuf file, containing its filename and source code.
pub(crate) struct ProtoGenFile {
    /// The name of the generated file.
    pub filename: String,
    /// The full source code content of the generated file.
    pub source_code: String,
}

impl ProtoGenFile {
    /// Creates a new `ProtoGenFile` by composing the header and content of a protobuf file.
    ///
    /// # Parameters
    /// - `filename`: The name of the file to be generated.
    /// - `package_name`: The package name for the protobuf definitions; it will be formatted to snake_case.
    /// - `header_comments`: A list of comments to include at the beginning of the file.
    /// - `imports`: An optional set of import paths to include in the file.
    /// - `content`: The main content (body) of the protobuf file.
    ///
    /// # Returns
    /// Returns a `ProtoGenFile` containing the filename and the composed source code.
    pub(crate) fn new(
        filename: String,
        package_name: &str,
        header_comments: &[String],
        imports: Option<HashSet<String>>,
        content: &str,
    ) -> Result<ProtoGenFile, fmt::Error> {
        let mut source_code = String::with_capacity(1024);
        write_header(
            &mut source_code,
            format_package_name(package_name),
            header_comments,
            imports,
        )?;
        writeln!(source_code, "{}", content)?;
        Ok(ProtoGenFile {
            filename,
            source_code,
        })
    }
}

/// Writes the header section of a protobuf file into the provided string buffer.
///
/// The header includes optional comments, the syntax declaration, package name, and import statements.
///
/// # Parameters
/// - `f`: A mutable reference to the string buffer where the header will be written.
/// - `package_name`: The formatted package name for the protobuf file.
/// - `header_comments`: A slice of header comment strings to include at the top of the file.
/// - `imports`: An optional set of import paths to include.
///
/// # Returns
/// A `fmt::Result` indicating success or failure of the write operations.
pub(crate) fn write_header(
    f: &mut String,
    package_name: String,
    header_comments: &[String],
    imports: Option<HashSet<String>>,
) -> fmt::Result {
    if !header_comments.is_empty() {
        for comment in header_comments {
            writeln!(f, "// {comment}")?;
        }
        f.push('\n');
    }

    writeln!(f, "syntax = \"proto3\";")?;
    f.push('\n');

    writeln!(f, "package {package_name};")?;
    f.push('\n');

    if let Some(imports) = imports {
        if !imports.is_empty() {
            for import in imports.iter().sorted() {
                writeln!(f, "import \"{import}\";")?;
            }
            f.push('\n');
        }
    }

    Ok(())
}

/// Formats a package name by converting each segment to snake_case.
///
/// # Parameters
/// - `package_name`: The package name in dot-separated notation.
///
/// # Example
///
/// ```
/// let formatted = format_package_name("Google.Protobuf.WellKnownTypes");
/// assert_eq!(formatted, "google.protobuf.well_known_types");
/// ```
pub(crate) fn format_package_name(package_name: &str) -> String {
    package_name
        .split('.')
        .map(|s| s.to_snake_case())
        .collect::<Vec<_>>()
        .join(".")
}

/// Formats a package filename from a package name by converting segments to snake_case.
///
/// # Parameters
/// - `package_name`: The package name in dot-separated notation.
///
/// # Example
///
/// ```
/// let filename = format_package_filename("google.protobuf");
/// assert_eq!(filename, "google/protobuf.proto");
/// ```
pub(crate) fn format_package_filename(package_name: &str) -> String {
    let package_name = package_name
        .split('.')
        .map(|s| s.to_snake_case())
        .collect::<Vec<_>>()
        .join("/");
    format!("{}.proto", package_name)
}

/// Writes an entry (root) protobuf file to the specified path.
///
/// The entry file includes the syntax declaration, package declaration, and public import statements.
///
/// # Parameters
/// - `file_path`: The file system path where the entry file will be created.
/// - `namespace`: The package namespace for the protobuf file.
/// - `public_imports`: A list of public import paths to be included in the file.
///
/// # Returns
/// An `io::Result<()>` which is `Ok` on success, or an error if file writing fails.
pub(crate) fn write_entry_file(
    file_path: PathBuf,
    namespace: &str,
    public_imports: Vec<String>,
) -> io::Result<()> {
    use std::io::Write;

    let mut file = File::create(file_path)?;

    // Write the syntax and package declarations.
    writeln!(file, "syntax = \"proto3\";\n")?;
    writeln!(file, "package {};\n", namespace)?;

    // Write public imports.
    for import in public_imports {
        writeln!(file, "import public \"{}\";", import)?;
    }

    Ok(())
}
