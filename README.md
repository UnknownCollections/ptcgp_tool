# PTCGP Utility Tool

A command-line or text-based user interface (TUI) multi-tool designed for interacting with PTCGP files, specifically targeting APKs, XAPKs, IL2CPP binaries, and metadata files.

## Features

- **Extract Protobuf Definitions**: Extract protobuf definitions from APK, XAPK, or IL2CPP metadata files.
- **Patch IL2CPP**: Patch IL2CPP binaries to remove modification detection by updating code hashes. Not fully tested.

<video src="docs/ptcgp_tool_demo.mp4"></video>

## Installation

### Prerequisites

- Rust (1.85 or later)

### Building from Source

```bash
git clone https://github.com/yourusername/ptcgp_tool.git
cd ptcgp_tool
cargo build --release
```

### Running

```bash
./target/release/ptcgp_tool [OPTIONS] <COMMAND>
```

## Usage

### CLI Mode

**Extract Protobuf Definitions:**

```bash
./ptcgp_tool --headless extract-proto --output <OUTPUT_DIR> [--apk <APK_PATH>] [--il2cpp <IL2CPP_PATH> --global-metadata <METADATA_PATH>] [--overwrite]
```

**Patch IL2CPP:**

```bash
./ptcgp_tool --headless patch <MODIFIED_OUTPUT_PATH> [--apk <APK_PATH>] [--il2cpp <IL2CPP_PATH>] [--global-metadata <METADATA_PATH>]
```

**Verbose Logging:**

```bash
./ptcgp_tool --verbose extract-proto --output <OUTPUT_DIR> [OPTIONS]
```

### TUI Mode

Launch the interactive TUI:

```bash
./ptcgp_tool
```

## Commands

### extract-proto

Extract protobuf definitions.

- `--apk <APK>`: Path to an APK file.
- `--il2cpp <IL2CPP>`: Path to the IL2CPP file.
- `--global-metadata <GLOBAL_METADATA>`: Path to the global-metadata file.
- `--output <OUTPUT>`: Output directory.
- `--overwrite`: Overwrite existing output.

### patch

Patch IL2CPP file hashes.

- `<MODIFIED>`: Path for the modified IL2CPP file.
- Optional paths to original APK, IL2CPP, and metadata files.

Use `--help` to display detailed command options:

```bash
./ptcgp_tool extract-proto --help
./ptcgp_tool patch --help
```

## Contributing

Contributions are welcome! Please open issues and pull requests according to the project's guidelines.

## License

This project is licensed under The Unlicense License. See [LICENSE](LICENSE) for details.
