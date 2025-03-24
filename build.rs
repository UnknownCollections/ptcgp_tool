use std::fs;
use std::path::Path;

/// Main entry point for the build script.
///
/// This function extracts the package version from Cargo.toml and constructs a full version string
/// by combining the major and minor version numbers with an incrementing build number. The build number
/// is read from and written to a "BUILD" file. The final version is then exported as an environment
/// variable for use during compilation.
fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src");

    let cargo_toml = "Cargo.toml";
    let build_number_file = Path::new("BUILD");

    // Read Cargo.toml and extract major.minor version
    let cargo_contents = fs::read_to_string(cargo_toml)
        .expect("Failed to read Cargo.toml");
    let version_line = cargo_contents
        .lines()
        .find(|line| line.starts_with("version = "))
        .expect("Failed to find version in Cargo.toml");

    let version = version_line
        .split('=')
        .nth(1)
        .expect("Invalid version format")
        .trim()
        .trim_matches('"'); // Extracts version string like "1.2.3"

    let major_minor: String = version
        .split('.')
        .take(2) // Get only major.minor parts
        .collect::<Vec<&str>>()
        .join(".");

    // Determine the new build number.
    // If the BUILD file exists, parse its content and increment; otherwise, start at 1.
    let build_number = if build_number_file.exists() {
        fs::read_to_string(build_number_file)
            .unwrap()
            .trim()
            .parse::<u32>()
            .unwrap_or(0)
            + 1
    } else {
        1
    };

    // Persist the updated build number back to the file.
    fs::write(build_number_file, build_number.to_string())
        .expect("Unable to write build number");

    // Generate the full version string in the format "major.minor.build".
    let full_version = format!("{}.{}", major_minor, build_number);

    // Export the version as an environment variable for use in the build.
    println!("cargo:rustc-env=VERSION={}", full_version);
}
