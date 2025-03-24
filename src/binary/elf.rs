use crate::binary::search::find_pattern;
use anyhow::{anyhow, bail, Result};
use goblin::elf32::program_header::{PF_W, PF_X, PT_LOAD};
use goblin::elf64::reloc;
use hashbrown::HashMap;
use log::debug;
use nohash_hasher::IntMap;
use std::hash::Hasher;
use std::mem::{size_of, transmute};
use std::ops::Range;

// Map relocation addends to a vector of target virtual addresses.
type RelocMap = IntMap<i64, Vec<u64>>;

// Define the pointer size for a 64-bit target.
pub const POINTER_SIZE: usize = size_of::<u64>();

/// Represents an ELF file with both its original and modified data, as well as metadata
/// (like section ranges and applied relocations). The `inner` field holds the parsed ELF.
pub struct Elf<'a> {
    pub inner: goblin::elf::Elf<'a>,
    // Modified file data (e.g. after relocations have been applied)
    pub data: Vec<u8>,
    // The original file data, before any modifications.
    pub original_data: Vec<u8>,
    // Mapping of relocation addends to their target virtual addresses.
    pub relocations: RelocMap,
    // Maps section names to their corresponding file data range.
    pub sections: HashMap<String, Range<usize>>,
}

impl<'a> Elf<'a> {
    /// Creates a new Elf instance by parsing the provided file data.
    ///
    /// This function:
    /// - Parses the raw ELF data.
    /// - Ensures that the ELF is 64-bit.
    /// - Applies dynamic relocations to update the file data.
    /// - Builds a map of section names to file ranges.
    /// - Transmutes the lifetime of the parsed ELF for internal use.
    pub fn new(data: Vec<u8>) -> Result<Elf<'a>> {
        debug!("Loading IL2CPP as ELF64...");

        let original_data = data;
        let elf = goblin::elf::Elf::parse(&original_data)?;

        if !elf.is_64 {
            bail!("Only 64-bit ELF files are supported");
        }

        debug!("Applying ELF dynamic relocations...");
        let (data, relocations) = Self::apply_dynamic_relocations(&elf, original_data.clone())?;

        // Build a mapping from section names to their file offsets.
        debug!("Building ELF section mapping...");
        let sections = Self::get_section_slices(&elf);

        // SAFETY: We transmute the lifetime of 'elf' to '`a'
        // because `original_data` will remain owned by this struct and will not be moved.
        let inner = unsafe { transmute::<goblin::elf::Elf<'_>, goblin::elf::Elf<'a>>(elf) };

        Ok(Elf {
            inner,
            data,
            original_data,
            relocations,
            sections,
        })
    }

    /// Applies dynamic relocations by iterating over each relocation entry
    /// and updating the in-memory file data.
    ///
    /// For each relocation, the code converts the target virtual address to a file offset,
    /// then writes the correct relocated value depending on the relocation type.
    pub fn apply_dynamic_relocations(
        elf: &goblin::elf::Elf,
        mut data: Vec<u8>,
    ) -> Result<(Vec<u8>, RelocMap)> {
        let mut relocs = RelocMap::default();
        for rela in &elf.dynrelas {
            let target_va = rela.r_offset;
            let addend = match rela.r_addend {
                Some(addend) => {
                    // Record the addend with its target address for later reference.
                    relocs.entry(addend).or_default().push(target_va);
                    addend
                }
                None => 0,
            };

            // Convert the target virtual address (VA) to a file offset.
            if let Some(file_offset) = Self::inner_va_to_file_offset(elf, target_va) {
                match rela.r_type {
                    reloc::R_AARCH64_RELATIVE => {
                        // For R_AARCH64_RELATIVE, the new value is (base address + addend).
                        // Here the base address is assumed to be zero so the relocation value equals the addend.
                        let bytes = (addend as u64).to_le_bytes();
                        data[file_offset as usize..file_offset as usize + 8]
                            .copy_from_slice(&bytes);
                    }

                    reloc::R_AARCH64_GLOB_DAT | reloc::R_AARCH64_JUMP_SLOT => {
                        // These relocations update the pointer to point to the resolved symbol.
                        let symbol_addr = Self::inner_resolve_symbol(elf, rela.r_sym)?;
                        let bytes = symbol_addr.to_le_bytes();
                        data[file_offset as usize..file_offset as usize + 8]
                            .copy_from_slice(&bytes);
                    }

                    reloc::R_AARCH64_ABS64 => {
                        // Absolute relocation: value = symbol address + addend.
                        let symbol_addr = Self::inner_resolve_symbol(elf, rela.r_sym)?;
                        let relocation_value = symbol_addr + addend as u64;
                        let bytes = relocation_value.to_le_bytes();

                        data[file_offset as usize..file_offset as usize + 8]
                            .copy_from_slice(&bytes);
                    }

                    _ => {
                        bail!(
                            "Unhandled relocation type: {} at 0x{:x}",
                            rela.r_type,
                            target_va
                        );
                    }
                }
            } else {
                bail!(
                    "Could not find file offset for relocation at VA 0x{:x}",
                    target_va
                );
            }
        }

        Ok((data, relocs))
    }

    /// Constructs a mapping of section names to their corresponding file offset ranges.
    ///
    /// This uses the section header string table to name each section.
    fn get_section_slices(elf: &goblin::elf::Elf) -> HashMap<String, Range<usize>> {
        elf.section_headers
            .iter()
            .filter_map(|section_hdr| {
                elf.shdr_strtab.get_at(section_hdr.sh_name).map(|name| {
                    let section_offset = section_hdr.sh_offset as usize;
                    let section_range =
                        section_offset..section_offset + section_hdr.sh_size as usize;
                    (name.to_string(), section_range)
                })
            })
            .collect()
    }

    /// Converts a virtual address (VA) into a file offset using the program headers.
    pub fn va_to_file_offset(&self, va: u64) -> Option<u64> {
        Self::inner_va_to_file_offset(&self.inner, va)
    }

    /// Iterates over the program headers to find the segment containing `va`
    /// and computes the corresponding file offset.
    fn inner_va_to_file_offset(elf: &goblin::elf::Elf, va: u64) -> Option<u64> {
        for ph in &elf.program_headers {
            let ph_start = ph.p_vaddr;
            let ph_end = ph.p_vaddr + ph.p_filesz;
            if ph_start <= va && va < ph_end {
                let offset_in_segment = va - ph.p_vaddr;
                return Some(ph.p_offset + offset_in_segment);
            }
        }
        None
    }

    /// Converts a file offset back to its corresponding virtual address.
    pub fn file_offset_to_va(&self, file_offset: u64) -> Option<u64> {
        Self::inner_file_offset_to_va(&self.inner, file_offset)
    }

    /// Iterates over the program headers to find the segment that includes `file_offset`
    /// and calculates the corresponding virtual address.
    fn inner_file_offset_to_va(elf: &goblin::elf::Elf, file_offset: u64) -> Option<u64> {
        for ph in &elf.program_headers {
            let ph_start = ph.p_offset;
            let ph_end = ph.p_offset + ph.p_filesz;

            if ph_start <= file_offset && file_offset < ph_end {
                let offset_in_segment = file_offset - ph.p_offset;
                return Some(ph.p_vaddr + offset_in_segment);
            }
        }
        None
    }

    /// Public interface to resolve a symbol address given its index.
    pub fn resolve_symbol(&self, sym_index: usize) -> Result<u64> {
        Self::inner_resolve_symbol(&self.inner, sym_index)
    }

    /// Looks up a symbol in the dynamic symbol table and returns its value (address).
    ///
    /// Returns an error if the symbol index is out of bounds.
    fn inner_resolve_symbol(elf: &goblin::elf::Elf, sym_index: usize) -> Result<u64> {
        let symbol = match elf.dynsyms.get(sym_index) {
            Some(sym) => sym,
            None => {
                bail!("Symbol not found for index: {}", sym_index);
            }
        };

        let resolved_address = symbol.st_value;

        Ok(resolved_address)
    }

    /// Reads a null-terminated string from a virtual address.
    ///
    /// If `len` is provided, at most that many bytes are read.
    pub fn read_va_str(&self, va: u64, len: Option<usize>) -> Result<String> {
        // Convert the virtual address to a file offset.
        let ra = self
            .va_to_file_offset(va)
            .ok_or(anyhow!("Invalid virtual address"))?;

        Ok(self.read_ra_str(ra, len))
    }

    /// Reads a string from the file data starting at file offset `ra`.
    ///
    /// If `len` is provided, exactly that many bytes are read;
    /// otherwise, bytes are read until a null terminator is encountered.
    pub fn read_ra_str(&self, ra: u64, len: Option<usize>) -> String {
        let s: String = if let Some(len) = len {
            // When a length is specified, read that many bytes.
            self.data.iter().take(len).map(|&c| c as char).collect()
        } else {
            // Otherwise, read until a null byte is found.
            self.data[ra as usize..]
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as char)
                .collect()
        };
        s
    }

    /// Checks whether the given virtual address is valid by ensuring it lies within
    /// a loadable segment that is marked as executable or writable.
    pub fn is_valid_pointer(&self, va: u64) -> bool {
        self.inner.program_headers.iter().any(|ph| {
            if ph.p_type == PT_LOAD {
                let is_executable = (ph.p_flags & PF_X) != 0;
                let is_writable = (ph.p_flags & PF_W) != 0;

                let in_range = ph.p_vaddr <= va && va < (ph.p_vaddr + ph.p_memsz);
                return in_range && (is_executable || is_writable);
            }
            false
        })
    }

    /// Reads an array of pointer-sized values from the given virtual address.
    ///
    /// Iterates `count` times, converting each pointer from file data using the VA-to-file-offset mapping.
    pub fn read_pointer_array(&self, va: u64, count: usize) -> Vec<u64> {
        let mut pointers = Vec::new();
        let mut current_va = va;

        for _ in 0..count {
            if let Some(file_offset) = self.va_to_file_offset(current_va) {
                if file_offset + POINTER_SIZE as u64 > self.data.len() as u64 {
                    break;
                }

                let ptr_bytes =
                    &self.data[file_offset as usize..file_offset as usize + POINTER_SIZE];
                let ptr_value = u64::from_le_bytes(ptr_bytes.try_into().unwrap());

                pointers.push(ptr_value);
                current_va += POINTER_SIZE as u64;
            } else {
                break;
            }
        }

        pointers
    }

    /// Find all locations in ELF sections where `pattern` appears.
    ///
    /// Searches each section for the given byte pattern and returns matching file offsets.
    pub fn search_elf(&self, pattern: &[u8]) -> Vec<u64> {
        let mut results = Vec::new();

        for shdr in &self.inner.section_headers {
            // Skip sections that have no data.
            if shdr.sh_size == 0 {
                continue;
            }
            let start = shdr.sh_offset as usize;
            let end = start + shdr.sh_size as usize;
            if end > self.data.len() {
                continue;
            }

            let section_bytes = &self.data[start..end];
            for result in find_pattern(section_bytes, pattern) {
                results.push((start + result) as u64);
            }
        }

        results
    }

    /// Reads `num_bytes` from the file data at the virtual address `va`, if that range is valid.
    pub fn read_bytes_at_va(&'a self, va: u64, num_bytes: usize) -> Option<&'a [u8]> {
        let file_offset = self.va_to_file_offset(va)?;
        let slice_end = file_offset as usize + num_bytes;
        if slice_end > self.data.len() {
            return None;
        }
        Some(&self.data[file_offset as usize..slice_end])
    }

    /// Searches for a byte sequence (`needle`) within specified ELF sections.
    ///
    /// Returns a vector of tuples containing the section name and the matching file offset.
    pub fn search_elf_sections(
        &self,
        section_names: &[&str],
        needle: &[u8],
    ) -> Result<Vec<(String, usize)>> {
        if needle.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        for (name, range) in &self.sections {
            if section_names.contains(&name.as_str()) {
                let data = &self.data[range.start..range.end];
                for result in find_pattern(data, needle) {
                    results.push((name.to_string(), range.start + result));
                }
            }
        }

        Ok(results)
    }

    /// Computes a hash for a given section's data using the provided hasher.
    ///
    /// This can be used for verifying section integrity or for caching purposes.
    pub fn hash_elf64_section<H: Hasher>(&self, segment_name: &str, mut hasher: H) -> Result<H> {
        let segment_range = self
            .sections
            .get(segment_name)
            .ok_or(anyhow!("No segment with name {segment_name} found"))?;

        let section_data = &self.data[segment_range.start..segment_range.end];
        hasher.write(section_data);

        Ok(hasher)
    }

    /// Consumes this Elf instance and returns a tuple of the modified file data and the original data.
    pub fn take(self) -> (Vec<u8>, Vec<u8>) {
        (self.data, self.original_data)
    }
}
