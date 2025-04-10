use crate::binary::elf::{Elf, POINTER_SIZE};
use crate::hash::il2cpp_code_hasher::{Il2CppPocketCodeHasher, Il2CppXorCodeHasher};
use crate::unity::il2cpp::Il2Cpp;
use anyhow::Result;
use hashbrown::HashMap;
use log::debug;
use parking_lot::Mutex;
use rayon::prelude::*;
use std::fmt::{Display, Formatter};
use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Represents a raw protected method entry in the binary.
///
/// This structure is used to read method metadata from the binary's data section.
/// The fields correspond to the method’s starting address, its size, and a stored hash.
#[repr(C)]
struct ProtectedMethodMetadata {
    /// Virtual address where the method's code starts.
    addr: u64,
    /// Size (in bytes) of the method's code.
    size: u64,
    /// Stored hash value for the method's code.
    hash: u64,
}

/// Enumerates the hash types used for verifying a protected method.
///
/// This distinguishes between an XOR-based hash and a pocket hash with a multiplier.
#[derive(Copy, Clone)]
pub enum ProtectedMethodHash {
    /// Indicates the hash is computed using XOR-based hashing.
    Xor,
    /// Indicates the hash is computed using a pocket hasher with the given multiplier.
    Pocket(u64),
}

impl Display for ProtectedMethodHash {
    /// Formats the `ProtectedMethodHash` as a human-readable string.
    ///
    /// The pocket variant displays the multiplier in hexadecimal.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtectedMethodHash::Xor => write!(f, "XOR"),
            ProtectedMethodHash::Pocket(v) => write!(f, "POCKET({v:#X})"),
        }
    }
}

/// Contains detailed metadata for a protected method.
///
/// This structure holds both the original method information and computed details
/// used to verify and update method code integrity.
#[derive(Clone)]
pub struct ProtectedMethodInfo {
    /// Virtual address of the method.
    pub addr: u64,
    /// Size (in bytes) of the method's code.
    pub size: u64,
    /// Expected hash value of the method's code.
    pub hash: u64,
    /// Type of hash used for this method.
    pub hash_type: ProtectedMethodHash,
    /// File offset address where the hash is stored in the binary.
    pub metadata_addr: u64,
    /// Name of the method, if known.
    pub name: String,
}

impl Display for ProtectedMethodInfo {
    /// Formats the `ProtectedMethodInfo` for display.
    ///
    /// The output includes the method’s address, hash type, name, size, stored hash, and metadata location.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:#X} {}: {} ({} bytes) = {:#X} @ {:#X}",
            self.addr, self.hash_type, self.name, self.size, self.hash, self.metadata_addr
        )
    }
}

/// Maximum allowed code size for a method (1 megabyte).
const MAX_CODE_SIZE: u64 = 1 << 20;

/// Scans the binary's data section to locate and validate protected methods.
///
/// This function iterates over the `.data` section of the ELF binary associated with the
/// `Il2Cpp` instance. It searches for potential `ProtectedMethodMetadata` entries, validates
/// their pointers and boundaries, computes the hash of the corresponding method code,
/// and if the computed hash matches the stored hash, adds the method to the results.
///
/// # Arguments
///
/// * `il2cpp` - A reference to the `Il2Cpp` instance containing the ELF binary and method metadata.
/// * `hash_constants` - A slice of constants used as multipliers when computing pocket hashes.
///
/// # Returns
///
/// A `Result` containing a vector of `ProtectedMethodInfo` entries if successful, or an error.
pub fn find_protected_fns<'a>(
    il2cpp: &'a Il2Cpp<'a>,
    hash_constants: &[u64],
) -> Result<Vec<ProtectedMethodInfo>> {
    // Retrieve the .data section range and slice from the ELF.
    let data_section_range = il2cpp.elf.sections.get(".data").unwrap();
    let data_section = &il2cpp.elf.data[data_section_range.start..data_section_range.end];
    let data_section_len = data_section.len();
    let file_data = &il2cpp.elf.original_data;
    let file_data_len = file_data.len() as u64;

    // Build a map from method virtual address to method name for known methods.
    let methods = il2cpp.methods()?;
    let method_map: HashMap<u64, &str> = methods
        .iter()
        .map(|(addr, name)| (*addr, name.as_str()))
        .collect();

    // Mutex-protected vector for collecting valid protected methods across threads.
    let results = Mutex::new(Vec::new());

    // Determine chunking parameters for parallel processing.
    // If more than one thread is available, use at least 2 chunks.
    let total_size = data_section.len();
    let num_threads = rayon::current_num_threads();
    let chunk_size = total_size / num_threads.min(2);

    // Compute total iterations for progress reporting.
    // We slide one byte at a time so each potential start is an iteration.
    let total_iterations = data_section.len().saturating_sub(POINTER_SIZE) + 1;
    let processed_iterations = AtomicUsize::new(0);

    // Log the start of processing.
    debug!(progress = 0, max = total_iterations; "");

    // Process the .data section in parallel by dividing it into chunks.
    data_section
        .par_chunks(chunk_size)
        .enumerate()
        .for_each(|(chunk_index, chunk_data)| {
            // Global offset of the current chunk within the .data section.
            let chunk_start = chunk_index * chunk_size;
            let mut local_hits = Vec::new();

            // Determine the number of sliding-window iterations in this chunk.
            let num_iterations = chunk_data.len().saturating_sub(POINTER_SIZE) + 1;
            for i in 0..num_iterations {
                let chunk_offset = chunk_start + i;
                // Read POINTER_SIZE bytes to extract a candidate pointer.
                let addr_bytes = &chunk_data[i..i + POINTER_SIZE];
                let addr = u64::from_le_bytes(addr_bytes.try_into().unwrap());

                // Check if the pointer is within valid bounds of the ELF.
                if il2cpp.elf.is_valid_pointer(addr) {
                    // Ensure there is enough room to read a complete ProtectedMethodMetadata.
                    if chunk_offset + size_of::<ProtectedMethodMetadata>() <= data_section_len {
                        // SAFETY: The pointer arithmetic is bounded by the earlier length check.
                        let pm = unsafe {
                            &*(data_section.as_ptr().add(chunk_offset)
                                as *const ProtectedMethodMetadata)
                        };

                        // Validate metadata fields:
                        // - Size must be non-zero and within maximum allowed code size.
                        // - The method's code (addr + size) must lie within the file data.
                        if pm.size != 0
                            && pm.size <= MAX_CODE_SIZE
                            && pm.addr + pm.size <= file_data_len
                        {
                            // Convert the method's virtual address to a file offset.
                            if let Some(file_offset) = il2cpp.elf.va_to_file_offset(pm.addr) {
                                let file_offset = file_offset as usize;
                                if file_offset + pm.size as usize <= file_data.len() {
                                    // Get the slice of method code from the file data.
                                    let method_data =
                                        &file_data[file_offset..file_offset + pm.size as usize];

                                    // Compute hash(es) based on the stored hash value.
                                    let mut hashes = vec![];
                                    if pm.hash <= u8::MAX as u64 {
                                        // Use XOR-based hash if the stored hash is small.
                                        hashes.push((
                                            Il2CppXorCodeHasher::hash(method_data),
                                            ProtectedMethodHash::Xor,
                                        ));
                                    } else {
                                        // Otherwise, try pocket hasher with each provided constant.
                                        for multiplier in hash_constants {
                                            hashes.push((
                                                Il2CppPocketCodeHasher::hash(
                                                    method_data,
                                                    *multiplier,
                                                ),
                                                ProtectedMethodHash::Pocket(*multiplier),
                                            ));
                                        }
                                    };

                                    // If any computed hash matches the stored hash, record the protected method.
                                    if let Some((_, hash_type)) =
                                        hashes.into_iter().find(|(hash, _)| *hash == pm.hash)
                                    {
                                        let method_name = method_map
                                            .get(&addr)
                                            .map(|name| name.to_string())
                                            .unwrap_or_else(|| format!("sub_{:x}", addr));

                                        local_hits.push(ProtectedMethodInfo {
                                            addr,
                                            size: pm.size,
                                            hash: pm.hash,
                                            hash_type,
                                            metadata_addr: (data_section_range.start + chunk_offset)
                                                as u64,
                                            name: method_name,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Update the global progress counter with the iterations processed in this chunk.
            let processed =
                processed_iterations.fetch_add(num_iterations, Ordering::Relaxed) + num_iterations;
            debug!(progress = processed; "Processed chunk {}", chunk_index);

            // Merge the local findings into the shared results vector.
            results.lock().extend(local_hits);
        });

    Ok(results.lock().drain(..).collect())
}

/// Updates function hashes in a modified ELF binary based on the original Il2Cpp metadata.
///
/// This function locates protected methods in the original `Il2Cpp` instance,
/// recomputes their hashes on the modified binary data, and updates the hash values
/// in the modified binary if discrepancies are found.
///
/// # Arguments
///
/// * `il2cpp` - A reference to the original `Il2Cpp` instance containing method metadata.
/// * `modified_il2cpp` - A mutable reference to the modified ELF binary whose hashes may be updated.
/// * `hash_constants` - A slice of constants used as multipliers for pocket hash computation.
///
/// # Returns
///
/// A `Result` indicating success or containing an error if the process fails.
pub fn update_fn_hashes<'a>(
    il2cpp: &'a Il2Cpp<'a>,
    modified_il2cpp: &mut Elf,
    hash_constants: &[u64],
) -> Result<()> {
    debug!("Scanning original IL2CPP data section for protected method signatures...");
    let protected_fns = find_protected_fns(il2cpp, hash_constants)?;

    let mut mismatched_fns = Vec::new();

    // Iterate over each protected function to verify its hash against the modified binary.
    debug!(progress = 0, max = protected_fns.len(); "");
    for protected_fn in &protected_fns {
        debug!("\t-Found: {}", protected_fn);
        // Convert the virtual address of the method to its corresponding file offset in the modified binary.
        if let Some(file_offset) = modified_il2cpp.va_to_file_offset(protected_fn.addr) {
            let file_offset = file_offset as usize;
            let modified_data = &modified_il2cpp.original_data;
            // Ensure the method's data lies within the bounds of the modified binary.
            if file_offset + protected_fn.size as usize <= modified_data.len() {
                let method_data =
                    &modified_data[file_offset..file_offset + protected_fn.size as usize];

                // Recompute the hash using the appropriate hasher based on the method's hash type.
                let actual_hash = match protected_fn.hash_type {
                    ProtectedMethodHash::Xor => Il2CppXorCodeHasher::hash(method_data),
                    ProtectedMethodHash::Pocket(multiplier) => {
                        Il2CppPocketCodeHasher::hash(method_data, multiplier)
                    }
                };

                // If the recomputed hash does not match the stored hash, mark it for update.
                if actual_hash != protected_fn.hash {
                    mismatched_fns.push((protected_fn.clone(), actual_hash));
                }
            }
        }
        debug!(progress_tick = 1; "");
    }

    // If mismatches were found, update the hash values in the modified binary.
    if !mismatched_fns.is_empty() {
        debug!("Updating hashes:");
        for (info, actual_hash) in mismatched_fns {
            // SAFETY: The update is safe because we ensure the metadata location is within bounds.
            let pm = unsafe {
                &mut *(modified_il2cpp
                    .original_data
                    .as_mut_ptr()
                    .add(info.metadata_addr as usize)
                    as *mut ProtectedMethodMetadata)
            };

            // If the hash has already been updated, skip further modification.
            if pm.hash == actual_hash {
                debug!("\t-{}: Already patched", info.name);
                continue;
            }
            debug!("\t-{}: {:#X} -> {:#X}", info.name, pm.hash, actual_hash);

            pm.hash = actual_hash;
        }
    }

    Ok(())
}
