use crate::binary::arm64::{
    parse_madd, parse_mov, parse_movk, parse_movn, parse_movz, Madd, Mov,
    Register, RET_INSTRUCTION_BYTES, SIZEOF_ARM64_INSTRUCTION,
};
use crate::binary::hex_pattern::HexPattern;
use crate::unity::il2cpp::Il2Cpp;
use anyhow::{bail, Result};
use elf::program_header;
use goblin::elf;
use log::debug;
use memchr::memmem::find;
use nohash_hasher::IntSet;
use rayon::prelude::*;

/// List of known constants to be ignored when scanning for function hash constants.
///
/// These constants are associated with known algorithms and libraries such as Brotli, Xxhash, and UnityEngine.
const KNOWN_CONSTANTS: [u64; 5] = [
    0xbd1e35a7bd000000, // Brotli
    0x35a7bd1e3fff0468, // Brotli
    0xbd1e35a71e35a7bd, // Brotli
    0xc2b2ae3d27d4eb4f, // Xxhash
    0x600000001,        // UnityEngine
];

/// Hex pattern corresponding to the sequence of instructions used in the segment hasher function.
///
/// This pattern is used to locate the start of the hasher function by matching a known sequence of ARM64 instructions.
const HASH_ACCUM_PATTERN: HexPattern = HexPattern::new(concat!(
    // 1) mov x8, #imm16
    "?? ?? ?? D2",
    // 2) mov x10, xzr
    " EA 03 1F AA",
    // 3) movk x8, #imm16, lsl #16
    " ?? ?? ?? F2",
    // 4) subs x9, x2, #4
    " 49 10 00 F1",
    // 5) movk x8, #imm16, lsl #32
    " ?? ?? ?? F2",
    // 6) movk x8, #imm16, lsl #48
    " ?? ?? ?? F2",
    // 7) b.eq ...
    " ?? ?? ?? 54",
    // 8) sub x11, x2, #5
    " 4B 14 00 D1",
    // 9) and x11, x11, #0xfffffffc
    " 6B F5 7E 92",
    // 10) ldr w12, [x1, x10]
    " 2C 68 6A B8",
    // 11) add x10, x10, #4
    " 4A 11 00 91",
    // 12) cmp x10, x9
    " 5F 01 09 EB",
    // 13) madd x0, x0, x8, x12
    " 00 30 08 9B",
    // 14) b.lo ...
    " ?? ?? ?? 54",
    // 15) add x10, x11, #4
    " 6A 11 00 91",
    // 16) cmp x10, x2
    " 5F 01 02 EB",
    // 17) b.hs ...
    " ?? ?? ?? 54",
    // 18) sub x9, x2, x10
    " 49 00 0A CB",
    // 19) add x10, x1, x10
    " 2A 00 0A 8B",
    // 20) ldrb w11, [x10], #1
    " 4B 15 40 38",
    // 21) subs x9, x9, #1
    " 29 05 00 F1",
    // 22) madd x0, x0, x8, x11
    " 00 2C 08 9B",
    // 23) b.ne ...
    " ?? ?? ?? 54"
));

/// Applies MOVK modifications onto a base immediate value.
///
/// Each MOVK instruction updates an entire 16-bit halfword of the base value.
/// The `movk_mask` parameter indicates which halfwords are modified, and `movk_value` holds the immediate values
/// to insert in their corresponding positions.
///
/// # Arguments
///
/// * `base` - The original 64-bit value to be modified.
/// * `movk_mask` - A 4-bit mask where each bit represents one 16-bit halfword to modify.
/// * `movk_value` - A 64-bit value that contains the new halfword values to be merged into `base`.
///
/// # Returns
///
/// The updated 64-bit value after applying all MOVK modifications.
#[inline]
fn apply_movk_modifications(base: u64, movk_mask: u8, movk_value: u64) -> u64 {
    let mut result = base;
    for i in 0..4 {
        if (movk_mask >> i) & 1 == 1 {
            let shift = i * 16;
            let half_mask: u64 = 0xFFFF << shift;
            // Clear the targeted halfword in `base` and insert the corresponding halfword from `movk_value`.
            result = (result & !half_mask) | (movk_value & half_mask);
        }
    }
    result
}

/// Attempts to reconstruct a constant value assigned to a register by scanning backward through instructions.
///
/// The function analyzes a slice of 32-bit instructions in reverse order, looking for MOVK, MOV, MOVZ, and MOVN
/// instructions that affect the specified register. It applies any found MOVK modifications on top of the base
/// value provided by a MOV, MOVZ, or MOVN instruction.
///
/// # Arguments
///
/// * `instructions` - A slice of 32-bit instruction words.
/// * `start_idx` - The starting index (from the end) in the instruction slice for backward scanning.
/// * `reg` - The target register whose constant value is to be reconstructed.
///
/// # Returns
///
/// * `Some(u64)` containing the reconstructed constant if successful.
/// * `None` if the constant cannot be reliably determined.
fn extract_reg_value(instructions: &[u32], start_idx: usize, reg: Register) -> Option<u64> {
    let mut movk_value = 0u64;
    let mut movk_mask = 0u8;
    let mut idx = start_idx;

    // Scan backwards instruction-by-instruction.
    while idx > 0 {
        idx -= 1;
        let inst = instructions[idx];

        // Check for MOVK: modifies individual 16-bit halfwords.
        if let Some(movk_inst) = parse_movk(inst) {
            if movk_inst.rd == reg {
                let shift = movk_inst.hw.to_shift_bits();
                let half = movk_inst.hw.to_u8();
                let half_mask = 0xFFFFu64 << shift;
                // Update movk_value by replacing the targeted halfword.
                movk_value = (movk_value & !half_mask) | ((movk_inst.imm16 as u64) << shift);
                movk_mask |= 1 << half;
                continue;
            }
        }

        // Check for MOV, which can be either a register-to-register move or a bitmask immediate move.
        if let Some(mov_inst) = parse_mov(inst) {
            if mov_inst.rd() == reg {
                let base = match mov_inst {
                    // For register-to-register MOV, treat it as setting the register to zero only if the source is XZR.
                    Mov::Register(mov) => {
                        if mov.rm == Register::Xzr {
                            0
                        } else {
                            return None;
                        }
                    }
                    // For bitmask immediate MOV, use the provided immediate value.
                    Mov::BitmaskImmediate(mov) => mov.imm(),
                };
                let final_val = apply_movk_modifications(base, movk_mask, movk_value);
                return Some(final_val);
            }
        }

        // Check for MOVZ: sets a 16-bit immediate into a zeroed register at a given halfword position.
        if let Some(movz_inst) = parse_movz(inst) {
            if movz_inst.rd == reg {
                let base = (movz_inst.imm16 as u64) << movz_inst.hw.to_shift_bits();
                let final_val = apply_movk_modifications(base, movk_mask, movk_value);
                return Some(final_val);
            }
        }

        // Check for MOVN: sets the register to the bitwise NOT of a shifted immediate.
        if let Some(movn_inst) = parse_movn(inst) {
            if movn_inst.rd == reg {
                let base = !((movn_inst.imm16 as u64) << movn_inst.hw.to_shift_bits());
                let final_val = apply_movk_modifications(base, movk_mask, movk_value);
                return Some(final_val);
            }
        }
    }

    None
}

/// Extracts immediate constant values from MADD instructions in a given instruction stream.
///
/// This function scans a slice of 32-bit ARM64 instructions in parallel to identify MADD instructions.
/// For each MADD instruction that passes the provided validation callback, it attempts to reconstruct the
/// immediate constant value from the register operand by scanning backwards using `extract_reg_value`.
///
/// # Type Parameters
///
/// * `F` - A closure type for validating a MADD instruction.
/// * `G` - A closure type for validating the reconstructed immediate constant.
///
/// # Arguments
///
/// * `instructions` - A slice of 32-bit ARM64 instruction words.
/// * `validate_madd` - A callback that returns `true` if a given MADD instruction meets the criteria.
/// * `validate_imm` - A callback that returns `true` if the reconstructed immediate constant is valid.
///
/// # Returns
///
/// A set of valid immediate constant values extracted from the instruction stream.
fn extract_madd_constants<F, G>(
    instructions: &[u32],
    validate_madd: F,
    validate_imm: G,
) -> IntSet<u64>
where
    F: Fn(&Madd) -> bool + Sync,
    G: Fn(u64) -> bool + Sync,
{
    instructions
        .par_iter()
        .enumerate()
        .fold(
            IntSet::default,
            |mut local_set, (i, &inst)| {
                // Check if the instruction is a MADD.
                if let Some(madd) = parse_madd(inst) {
                    // Use the provided callback to validate the MADD instruction.
                    if validate_madd(&madd) {
                        // Attempt to reconstruct the constant from the register used in the MADD.
                        if let Some(imm) = extract_reg_value(instructions, i, madd.rm) {
                            // Use the provided callback to validate the immediate value.
                            if validate_imm(imm) {
                                local_set.insert(imm);
                            }
                        }
                    }
                }
                local_set
            },
        )
        .reduce(
            IntSet::default,
            |mut a, b| {
                a.extend(b);
                a
            },
        )
}

/// Searches executable LOAD segments of an IL2Cpp binary for potential function hash constants.
///
/// This function iterates over executable segments (marked by PT_LOAD and PF_X) in the ELF binary,
/// converts the raw data into 32-bit ARM64 instructions, and then extracts constants from valid MADD
/// instructions. It ignores constants known to be associated with specific libraries or algorithms.
///
/// # Arguments
///
/// * `il2cpp` - A reference to the IL2Cpp binary, which includes ELF metadata and raw data.
///
/// # Returns
///
/// A vector of unique immediate constant values that are potential function hash constants.
///
/// # Errors
///
/// Returns an error if there is an issue processing the binary.
pub fn find_function_hash_constants(il2cpp: &Il2Cpp) -> Result<Vec<u64>> {
    let mut global_results = IntSet::default();

    // Filter and collect all LOAD segments that are marked as executable.
    let executable_segments: Vec<_> = il2cpp
        .elf
        .inner
        .program_headers
        .iter()
        .filter(|ph| {
            ph.p_type == program_header::PT_LOAD && (ph.p_flags & program_header::PF_X) != 0
        })
        .collect();

    debug!("Scanning executable segments for function hash constants...");
    debug!(progress = 0, max = executable_segments.len(); "");

    for ph in executable_segments {
        debug!(
            "Processing segment at offset: {} with size: {}",
            ph.p_offset, ph.p_filesz
        );

        // Extract the executable segment data based on file offset and size.
        let data = &il2cpp.elf.data[ph.p_offset as usize..(ph.p_offset + ph.p_filesz) as usize];

        // Convert raw bytes into a vector of u32 instructions (assuming little-endian encoding).
        let instructions: Vec<u32> = data
            .chunks_exact(SIZEOF_ARM64_INSTRUCTION)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();

        // Merge the per-segment results into the global results.
        global_results.extend(extract_madd_constants(
            &instructions,
            |madd: &Madd| madd.sf == 1 && madd.rd == madd.ra,
            |imm: u64| {
                !KNOWN_CONSTANTS.contains(&imm)
                    && imm > 0xFFFFFFFF
                    && ((imm >> 32) != 0xFFFFFFFF)
                    && ((imm & 0xFFFFFFFF) != 0)
                    && ((imm & 0xFFFFFFFF) != 0xFFFFFFFF)
            },
        ));

        // Increment progress after each segment is processed.
        debug!(progress_tick = 1; "");
    }

    debug!(
        "Finished scanning. Found {} unique constants.",
        global_results.len()
    );
    // Convert the set of constants to a vector for the final output.
    Ok(global_results.into_iter().collect())
}

/// Locates and extracts the function hash constant from the segment hasher function.
///
/// This function searches the ".text" section of an IL2Cpp binary for a specific sequence of bytes defined
/// by `HASH_ACCUM_PATTERN`. Once the start of the function is located, it determines the function's end by
/// finding the return instruction bytes. The contained ARM64 instructions are then parsed to extract valid
/// MADD constants.
///
/// # Arguments
///
/// * `il2cpp` - A reference to the IL2Cpp binary, including its ELF data and section mappings.
///
/// # Returns
///
/// The first extracted hash constant if found.
///
/// # Errors
///
/// Returns an error if the start or end of the segment hasher function cannot be located,
/// or if no valid MADD constants are found.
pub fn find_segment_hash(il2cpp: &Il2Cpp) -> Result<u64> {
    debug!("Scanning .text section for segment hash...");
    let text_section = &il2cpp.elf.sections[".text"];
    let data = &il2cpp.elf.data[text_section.start..text_section.end];

    // Step 1: Find the start offset.
    let start_offset = match HASH_ACCUM_PATTERN.find(data) {
        None => bail!("segment hasher function start offset not found"),
        Some(o) => {
            debug!("Found start offset: {}", o);
            o
        }
    };

    // Step 2: Find the end offset.
    let end_offset = match find(&data[start_offset..], &RET_INSTRUCTION_BYTES) {
        None => bail!("segment hasher function end offset not found"),
        Some(eo) => {
            let end_offset = start_offset + eo + SIZEOF_ARM64_INSTRUCTION;
            debug!("Found end offset: {}", end_offset);
            end_offset
        }
    };

    // Step 3: Extract function instructions.
    let func_data = &data[start_offset..end_offset];
    debug!(
        "Extracting instructions from function data (length: {})",
        func_data.len()
    );
    let instructions: Vec<u32> = func_data
        .chunks_exact(SIZEOF_ARM64_INSTRUCTION)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .collect();

    // Step 4: Extract madd constants.
    let constants = extract_madd_constants(&instructions, |_| true, |_| true);
    debug!("Found {} madd constants", constants.len());

    // Return the first constant, or bail if none were found.
    if let Some(first) = constants.iter().next() {
        debug!("Returning segment hash constant: {:#X}", first);
        Ok(*first)
    } else {
        bail!("no madd constants found in segment hasher function");
    }
}
