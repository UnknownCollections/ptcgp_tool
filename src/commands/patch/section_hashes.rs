use crate::binary::elf::Elf;
use crate::hash::il2cpp_code_hasher::Il2CppPocketCodeHasher;
use crate::unity::il2cpp::Il2Cpp;
use anyhow::{bail, Result};
use log::debug;
use std::hash::Hasher;

/// Computes the hash for specified sections of an ELF binary using a custom SHA-1 based hasher.
///
/// The function initializes a pocket code hasher with a constant value and iterates over each
/// provided section name. For each section, it updates the hasher using the computed section hash
/// from the ELF file. Once all sections have been processed, the final hash is returned.
///
/// # Parameters
/// - `elf`: A reference to an `Elf` object representing the ELF binary.
/// - `section_names`: A slice of section names whose data will be incorporated into the hash.
/// - `section_hash_constant`: A constant value used to initialize the custom hash algorithm,
///    which can influence the final hash output in a controlled manner.
///
/// # Returns
/// An `anyhow::Result` containing the computed hash as a `u64` if successful, or an error otherwise.
fn hash_elf(elf: &Elf, section_names: &[&str], section_hash_constant: u64) -> Result<u64> {
    // Initialize the custom hasher with the provided constant.
    let mut hasher = Il2CppPocketCodeHasher::new(section_hash_constant);
    for section_name in section_names {
        // Update the running hash with the computed hash for the given section.
        // The call to `hash_elf64_section` processes the section data and updates the hasher.
        hasher = elf.hash_elf64_section(section_name, hasher)?;
    }

    // Finalize and return the computed hash.
    Ok(hasher.finish())
}

/// Updates the hash within a modified ELF binary to reflect changes in code segments.
///
/// This function computes hashes for designated segments from both the original and modified ELF binaries.
/// If the computed hashes differ, the function searches for the original hash value in the modified binary’s
/// `.data` section and patches it with the new hash value. This operation helps ensure the integrity
/// signature of the modified file is up to date.
///
/// # Parameters
/// - `il2cpp`: A reference to an `Il2Cpp` object containing the original ELF binary.
/// - `modified_il2cpp`: A mutable reference to an `Elf` object representing the modified ELF binary.
/// - `segments`: A vector of segment names whose contents are used to compute the hash.
/// - `section_hash_constant`: A constant value used for the hash computation; the same constant should
///    be used for both original and modified ELF binaries.
///
/// # Returns
/// A `Result` indicating success (`Ok(())`) or an error detailing why the update failed.
///
/// # Errors
/// Returns an error if:
/// - The hash computation fails for either the original or modified ELF binary.
/// - The original hash is not found in the modified binary’s `.data` section.
/// - Multiple occurrences of the original hash are found in the modified binary.
pub fn update_section_hash(
    il2cpp: &Il2Cpp,
    modified_il2cpp: &mut Elf,
    segments: Vec<&str>,
    section_hash_constant: u64,
) -> Result<()> {
    // Compute the hash for the specified segments from the original ELF binary.
    let original_hash = hash_elf(&il2cpp.elf, &segments, section_hash_constant)?;
    debug!("Original section hash: {:#X}", original_hash);

    // Compute the hash for the same segments from the modified ELF binary.
    let new_hash = hash_elf(modified_il2cpp, &segments, section_hash_constant)?;
    // NOTE: The debug message below prints the original hash again. Verify whether this should be new_hash.
    debug!("Modified section hash: {:#X}", original_hash);

    // If no changes have been detected, no further action is required.
    if original_hash == new_hash {
        debug!("Hashes match, no update necessary");
        return Ok(());
    }

    // Convert the computed hash values into little-endian byte arrays.
    let original_hash_bytes = original_hash.to_le_bytes();
    let new_hash_bytes = new_hash.to_le_bytes();

    // Search for the occurrence of the original hash bytes within the '.data' section.
    // It is assumed that the original binary's layout is similar enough to the modified one for the offset to be valid.
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

    // Read the hash value at the found offset in the modified ELF binary's data.
    let modified_file_coded_hash = u64::from_le_bytes(
        modified_il2cpp.original_data
            [found_hash_offset..found_hash_offset + original_hash_bytes.len()]
            .try_into()?,
    );

    // Check if the modified file has already been patched with the new hash.
    if modified_file_coded_hash == new_hash {
        debug!("Modified IL2CPP file has already been patched...");
        return Ok(());
    }

    // Replace the original hash with the new hash bytes in the modified ELF binary.
    modified_il2cpp.original_data[found_hash_offset..found_hash_offset + original_hash_bytes.len()]
        .copy_from_slice(&new_hash_bytes);
    debug!("Patched section hash to: {:#X}", new_hash);

    Ok(())
}
