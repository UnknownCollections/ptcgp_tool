use std::fs;
use std::path::Path;

fn main() {
    let cargo_toml = "Cargo.toml";
    let build_number_file = Path::new("BUILD");

    // Read Cargo.toml and extract major.minor version
    let cargo_contents = fs::read_to_string(cargo_toml).expect("Failed to read Cargo.toml");
    let version_line = cargo_contents
        .lines()
        .find(|line| line.starts_with("version = "))
        .expect("Failed to find version in Cargo.toml");

    let version = version_line
        .split('=')
        .nth(1)
        .expect("Invalid version format")
        .trim()
        .trim_matches('"'); // Extracts "1.2.3"

    let major_minor: String = version
        .split('.')
        .take(2) // Get only major.minor
        .collect::<Vec<&str>>()
        .join(".");

    // Read and increment the build number
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

    // Write the new build number back to the file
    fs::write(build_number_file, build_number.to_string()).expect("Unable to write build number");

    // Generate full version
    let full_version = format!("{}.{}", major_minor, build_number);

    // Export it as an environment variable
    println!("cargo:rustc-env=VERSION={}", full_version);
}
