use std::fs;
use std::path::Path;

/// Updates the build number and exports a full version string as an environment variable.
///
/// This function first checks for a "SKIP_BUILD_NUMBER" environment variable to allow bypassing
/// the update. It then reads the `Cargo.toml` file to extract the major and minor version numbers
/// from the version string (e.g., "1.2.3" becomes "1.2"). Next, it checks for an existing "BUILD"
/// file; if found, it reads and increments the stored build number, otherwise it starts from 1.
/// The new build number is saved back to the "BUILD" file, and a full version string in the
/// format "major.minor.build" is constructed and exported for use during the build process.
fn update_build_number() {
    let cargo_toml = "Cargo.toml";
    let build_number_file = Path::new("BUILD");

    // Read the contents of Cargo.toml.
    let cargo_contents = fs::read_to_string(cargo_toml)
        .expect("Failed to read Cargo.toml");

    // Locate the line that specifies the version, e.g., `version = "1.2.3"`.
    let version_line = cargo_contents
        .lines()
        .find(|line| line.starts_with("version = "))
        .expect("Failed to find version in Cargo.toml");

    // Extract the version string from the line and remove surrounding quotes.
    let version = version_line
        .split('=')
        .nth(1)
        .expect("Invalid version format")
        .trim()
        .trim_matches('"'); // e.g., converts `"1.2.3"` to `1.2.3`

    // Get the major and minor parts of the version (ignore the patch version).
    let major_minor: String = version
        .split('.')
        .take(2) // Use only the first two components
        .collect::<Vec<&str>>()
        .join(".");

    // Read the current build number if the BUILD file exists; otherwise, start at 1.
    let mut build_number = if build_number_file.exists() {
        fs::read_to_string(build_number_file)
            .unwrap()
            .trim()
            .parse::<u32>()
            .unwrap_or(0)
    } else {
        1
    };

    // If the "SKIP_BUILD_NUMBER" variable is not set, update the build number.
    if std::env::var("SKIP_BUILD_NUMBER").is_err() {
        build_number += 1;
        fs::write(build_number_file, build_number.to_string())
            .expect("Unable to write build number");
    }

    // Construct the full version string by appending the build number.
    let full_version = format!("{}.{}", major_minor, build_number);

    // Export the full version string as an environment variable for the compiler.
    println!("cargo:rustc-env=VERSION={}", full_version);
}


/// Main entry point for the build script.
///
/// This function instructs Cargo to re-run the build script when certain files change, ensuring
/// that changes to the Cargo.toml or source files trigger a rebuild. It then calls the
/// `update_build_number` function to manage the versioning process.
fn main() {
    // Instruct Cargo to rerun this build script if Cargo.toml or any file in the src directory changes.
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src");

    update_build_number();
}
