#!/usr/bin/env python3

# Adapted from https://github.com/nneonneo/Il2CppVersions

import subprocess
from pathlib import Path


def process_il2cpp_dir(path: Path) -> None:
    try:
        version = path.name.split('il2cpp-', 1)[1]
    except IndexError:
        return  # Skip if the directory name doesn't match the expected format

    il2cpp_dir = path / 'libil2cpp'
    header_file = il2cpp_dir / 'il2cpp-object-internals.h'

    # Build the clang command arguments
    args = [
        'clang',
        '-E',
        '-x', 'c++',
        '--target=aarch64-linux-android',
        '-m64',
        '-P',
        '-D', '__ANDROID__',
        '-D', '__aarch64__',
        '-I', 'dummy',
        '-I', str(il2cpp_dir),
        '-include', str(il2cpp_dir / 'vm' / 'GlobalMetadataFileInternals.h'),
        '-include', str(il2cpp_dir / 'vm' / 'MemoryInformation.h'),
        str(header_file),
    ]

    try:
        print(f"Processing {version} with args: {' '.join(args)}")
        # Use encoding to avoid manual decoding
        header_output = subprocess.check_output(args, stderr=subprocess.PIPE, encoding='utf-8')
    except subprocess.CalledProcessError as e:
        print(f"Error processing {version}:")
        print(f"Command: {' '.join(e.cmd)}")
        print(f"Return code: {e.returncode}")
        print(f"Stderr: {e.stderr}")
        print(f"Stdout: {e.output}")
        return
    except FileNotFoundError:
        print("Error: 'clang' command not found. Is a C compiler (like gcc or clang) installed and in PATH?")
        exit(1)

    output_filename = f"{version}.h"
    with open(output_filename, 'w', encoding='utf-8') as f:
        f.write(header_output)
    print(f"Successfully generated {output_filename}")


def main() -> None:
    # Process each directory matching the pattern
    for path in Path('.').glob('il2cpp-*'):
        if path.is_dir():
            process_il2cpp_dir(path)


if __name__ == '__main__':
    main()
