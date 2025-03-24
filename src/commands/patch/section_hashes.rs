use crate::binary::elf::Elf;
use crate::hash::il2cpp_code_hasher::Il2CppPocketCodeHasher;
use crate::unity::il2cpp::Il2Cpp;
use anyhow::{bail, Result};
use log::debug;
use std::hash::Hasher;

/// Computes the hash for specified sections of an ELF binary using a custom SHA-1 based hasher.
///
/// This function iterates over the provided section names and updates the hasher with each section's
/// hash value from the ELF file. The final hash is computed once all sections have been processed.
///
/// # Parameters
/// - `elf`: A reference to an `Elf` object representing the ELF binary.
/// - `section_names`: A slice of section names whose data will be incorporated into the hash.
///
/// # Returns
/// An `io::Result` containing the computed hash as a `u64` if successful, or an error otherwise.
fn hash_elf(elf: &Elf, section_names: &[&str]) -> Result<u64> {
    let mut hasher = Il2CppPocketCodeHasher::new();
    for section_name in section_names {
        // Update the hasher using the hash of the specified ELF section.
        hasher = elf.hash_elf64_section(section_name, hasher)?;
    }

    Ok(hasher.finish())
}

/// Updates the hash within a modified ELF binary to validate any modifications done to code sections
///
/// This function computes hashes for designated segments from both the original and modified ELF binaries.
/// If the hashes differ, it locates the original hash within the modified binary's `.data` section and
/// replaces it with the new hash. This ensures the modified file reflects the expected integrity signature.
///
/// # Parameters
/// - `il2cpp`: A reference to an `Il2Cpp` object containing the original ELF binary.
/// - `modified_il2cpp`: A mutable reference to an `Elf` object representing the modified ELF binary.
/// - `segments`: A vector of segment names whose contents are used to compute the hash.
///
/// # Returns
/// A `Result` indicating success (`Ok(())`) or an error message encapsulated in a boxed error.
///
/// # Errors
/// Returns an error if:
/// - The hash computation fails for either the original or modified ELF binary.
/// - The original hash is not found in the modified binary.
/// - Multiple occurrences of the original hash are found in the modified binary.
pub fn update_section_hash(
    il2cpp: &Il2Cpp,
    modified_il2cpp: &mut Elf,
    segments: Vec<&str>,
) -> Result<()> {
    // Compute the hash from the original file's segments.
    let original_hash = hash_elf(&il2cpp.elf, &segments)?;
    debug!("Original section hash: {:#X}", original_hash);

    // Compute the hash from the modified file's segments.
    let new_hash = hash_elf(modified_il2cpp, &segments)?;
    debug!("Modified section hash: {:#X}", original_hash);

    // If the hashes match, no update is necessary.
    if original_hash == new_hash {
        debug!("Hashes match, no update necessary");
        return Ok(());
    }

    // Convert the u64 hash values into 8-byte arrays in little-endian format.
    let original_hash_bytes = original_hash.to_le_bytes();
    let new_hash_bytes = new_hash.to_le_bytes();

    // Search for the original hash bytes within the '.data' section of the modified ELF binary.
    let found = il2cpp
        .elf
        .search_elf_sections(&[".data"], &original_hash_bytes)?;
    if found.is_empty() {
        bail!("Original hash not found");
    }
    if found.len() > 1 {
        bail!("Too many matches for original hash");
    }
    let (_, found_hash_offset) = found[0];
    debug!("Found original hash at offset: {:#X}", found_hash_offset);

    let modified_hash = hash_elf(modified_il2cpp, &segments)?;

    let modified_file_coded_hash = u64::from_le_bytes(
        modified_il2cpp.original_data
            [found_hash_offset..found_hash_offset + original_hash_bytes.len()]
            .try_into()?,
    );

    if modified_file_coded_hash == modified_hash {
        debug!("Modified IL2CPP file has already been patched...");
        return Ok(());
    }

    // Replace the located original hash with the new hash bytes.
    modified_il2cpp.original_data[found_hash_offset..found_hash_offset + original_hash_bytes.len()]
        .copy_from_slice(&new_hash_bytes);
    debug!("Patched section hash to: {:#X}", modified_hash);

    Ok(())
}
