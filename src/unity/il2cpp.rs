use crate::binary::arm64::{
    parse_add_immediate, parse_adrp, parse_bl, parse_movk, parse_movz, Register,
    ShiftAmount, SIZEOF_ARM64_INSTRUCTION,
};
use crate::binary::elf::{Elf, POINTER_SIZE};
use crate::unity::generated::CIl2Cpp::{
    Il2CppCodeGenModule, Il2CppCodeRegistration, Il2CppMetadataRegistration, Il2CppType,
    Il2CppTypeEnum, IL2CPP_TYPE_ENUM,
};
use crate::unity::global_metadata::Metadata;
use crate::utils::read_only::ReadOnly;
use anyhow::{anyhow, bail, Result};
use hashbrown::HashMap;
use itertools::Itertools;
use log::debug;
use nohash_hasher::IntMap;
use std::io::Cursor;
use std::mem::{offset_of, size_of};

/// Represents the IL2CPP environment extracted from an ELF binary and its associated metadata.
///
/// This struct holds the ELF binary, the Unity metadata, and various registration structures
/// needed for IL2CPP analysis.
pub struct Il2Cpp<'a> {
    /// The ELF binary containing IL2CPP code.
    pub elf: Elf<'a>,
    /// Global metadata extracted from the Unity binary.
    pub metadata: Metadata,
    /// Code registration information for IL2CPP functions.
    code_registration: ReadOnly<Il2CppCodeRegistration>,
    /// Metadata registration information for IL2CPP types.
    metadata_registration: ReadOnly<Il2CppMetadataRegistration>,
    /// List of IL2CPP type definitions.
    pub types: Vec<ReadOnly<&'a Il2CppType>>,
    /// Mapping from type pointer addresses to indices in the `types` vector.
    type_ptr_map: IntMap<u64, usize>,
}

impl<'a> Il2Cpp<'a> {
    /// Loads the IL2CPP environment from the given binary data vectors.
    ///
    /// This function constructs an `Il2Cpp` instance by parsing the IL2CPP ELF binary and associated
    /// global metadata. It verifies the metadata version and loads both code and metadata registration structures.
    ///
    /// # Arguments
    ///
    /// * `elf` - An instance of a parsed IL2CPP ELF binary.
    /// * `global_metadata_data` - A vector of bytes representing the global metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata version is unsupported or if parsing fails.
    pub fn load_from_vec(elf: Elf<'a>, global_metadata_data: Vec<u8>) -> Result<Self> {
        debug!("Loading IL2CPP from ELF and metadata...");
        let reader = Cursor::new(global_metadata_data);
        let metadata = Metadata::load_from_reader(reader)?;

        let code_registration = Self::find_code_registration(&elf, &metadata)?;
        let metadata_registration = Self::find_metadata_registration(&elf, &metadata)?;

        debug!("Loading types from metadata registration...");
        let types = Self::inner_load_data_array::<Il2CppType>(
            &elf,
            metadata_registration.types,
            metadata_registration.typesCount as usize,
        )?
        .into_iter()
        .map(ReadOnly::new)
        .collect_vec();

        debug!("Loading type pointer map...");
        let type_ptr_map = elf
            .read_pointer_array(
                metadata_registration.types as u64,
                metadata_registration.typesCount as usize,
            )
            .into_iter()
            .enumerate()
            .map(|(idx, ptr)| (ptr, idx))
            .collect::<IntMap<_, _>>();

        Ok(Il2Cpp {
            elf,
            metadata,
            code_registration,
            metadata_registration,
            types,
            type_ptr_map,
        })
    }

    /// Extracts the metadata key's xor key from ARM64 instructions in the provided elf file data.
    ///
    /// This function scans through the provided data and looks for a sequence of instructions
    /// that match a specific pattern. When found, the immediate values from these instructions
    /// are combined into a 64-bit metadata key.
    ///
    /// # Arguments
    ///
    /// * `elf` - An instance of the Elf binary class
    ///
    /// # Returns
    ///
    /// Returns `Some((usize, u64))` containing the offset and metadata key if the pattern is found, or `None` otherwise.
    pub fn extract_metadata_key_xor(elf: &Elf) -> Option<(usize, u64)> {
        debug!("Extracting global metadata key xor data from IL2CPP...");

        let text_section_range = elf.sections.get(".text")?;
        let text_section_data = &elf.data[text_section_range.start..text_section_range.end];

        // ARM64 instructions are 4 bytes in little-endian order.
        let instructions: Vec<u32> = text_section_data
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        let total = instructions.len().saturating_sub(5);
        debug!(progress = 0, max = total; "");

        // Look for five consecutive instructions that meet the required criteria.
        for (i, window) in instructions.windows(5).enumerate() {
            debug!(progress_tick = 1; "");
            let inst1 = match parse_movz(window[0]) {
                Some(inst) => inst,
                None => continue,
            };
            if inst1.rd != Register::X1 {
                continue;
            }
            let inst2 = match parse_movk(window[2]) {
                Some(inst) => inst,
                None => continue,
            };
            if inst2.rd != Register::X1 || inst2.hw != ShiftAmount::Lsl16 {
                continue;
            }
            let inst3 = match parse_movk(window[3]) {
                Some(inst) => inst,
                None => continue,
            };
            if inst3.rd != Register::X1 || inst3.hw != ShiftAmount::Lsl32 {
                continue;
            }
            let inst4 = match parse_movk(window[4]) {
                Some(inst) => inst,
                None => continue,
            };
            if inst4.rd != Register::X1 || inst4.hw != ShiftAmount::Lsl48 {
                continue;
            }
            let combined = ((inst4.imm16 as u64) << 48)
                | ((inst3.imm16 as u64) << 32)
                | ((inst2.imm16 as u64) << 16)
                | inst1.imm16 as u64;

            // Calculate the offset in bytes (each instruction is 4 bytes).
            let offset = i * SIZEOF_ARM64_INSTRUCTION;
            return Some((offset, combined));
        }
        None
    }

    /**
     * Extracts a 16-byte metadata key from the provided ELF binary data using a predefined hex pattern.
     *
     * This function scans the .text section of the ELF for a specific pattern corresponding to a key XOR instruction.
     * Starting from the given key XOR offset, it searches for a BL (branch with link) instruction to locate a code region
     * that contains an ADRP instruction followed immediately by an ADD immediate instruction using the same destination register.
     * It then computes the key's virtual address by combining the ADRP’s page base with its immediate offset and the ADD’s immediate value,
     * converts that address to a file offset, and finally extracts the 16-byte key from the ELF data.
     *
     * # Arguments
     *
     * * `elf` - A reference to an `Elf` structure representing the ELF binary.
     * * `key_xor_offset` - The offset within the .text section pointing to the key XOR instruction.
     *
     * # Returns
     *
     * * `Some([u8; 16])` if the key is successfully extracted.
     * * `None` if the key is not found or if any computation fails.
     */
    pub fn extract_metadata_key(elf: &Elf, key_xor_offset: usize) -> Option<[u8; 16]> {
        debug!("Extracting global metadata encryption keys from IL2CPP...");

        // Retrieve the file offset range of the .text section and its corresponding instruction stream.
        let text_section_range = elf.sections.get(".text")?;
        let text_instructions = elf.instructions.get(".text")?;

        // Convert the key_xor_offset from a byte offset to an instruction index by dividing by the instruction size.
        let key_xor_inst_offset = key_xor_offset / SIZEOF_ARM64_INSTRUCTION;

        // Starting from the key XOR instruction index, iterate over instructions to locate a BL (branch with link) instruction.
        // The BL instruction contains a relative offset that, when combined with the current index,
        // gives us an adjusted instruction offset used to pinpoint the following ADRP/ADD instruction pair.
        let bl_inst_offset = text_instructions[key_xor_inst_offset..]
            .iter()
            .enumerate()
            .find_map(|(offset, &inst)| {
                parse_bl(inst).map(|bl| {
                    // Calculate the absolute offset by summing:
                    // - The offset within the slice,
                    // - The starting key XOR instruction index,
                    // - The BL instruction's embedded relative offset (converted to instruction units).
                    offset as i64
                        + key_xor_inst_offset as i64
                        + (bl.offset / SIZEOF_ARM64_INSTRUCTION as i64)
                })
            })? as usize;

        // Iterate over adjacent instruction pairs starting from the BL instruction offset.
        // We are looking for an ADRP instruction immediately followed by an ADD immediate instruction.
        // The ADD must use the destination register specified by the ADRP instruction.
        for (i, window) in text_instructions[bl_inst_offset..].windows(2).enumerate() {
            if let (Some(adrp), Some(add)) = (parse_adrp(window[0]), parse_add_immediate(window[1]))
            {
                if add.rn == adrp.rd {
                    // Calculate the absolute index of the ADRP instruction within the .text section.
                    let adrp_index = bl_inst_offset + i;
                    // Determine the file offset for the ADRP instruction.
                    let adrp_address =
                        text_section_range.start + adrp_index * SIZEOF_ARM64_INSTRUCTION;
                    // Convert the file offset to a virtual address.
                    let adrp_address_va = elf.file_offset_to_va(adrp_address as u64)?;
                    // Clear the lower 12 bits to obtain the ADRP page base, since ADRP instructions work on page granularity.
                    let page_base = adrp_address_va & !0xfff;
                    // Compute the final virtual address for the key by adding the ADRP and ADD immediate offsets to the page base.
                    let key_offset_va =
                        ((page_base as i64) + adrp.compute_imm() + add.immediate() as i64) as u64;
                    // Convert the computed virtual address back to a file offset.
                    let key_offset = elf.va_to_file_offset(key_offset_va)?;

                    // Extract and return a 16-byte slice from the ELF data as the metadata key.
                    return elf
                        .data
                        .get(key_offset as usize..key_offset as usize + 16)
                        .and_then(|slice| slice.try_into().ok());
                }
            }
        }

        // If no valid ADRP/ADD pair is found, return None.
        None
    }

    /// Locates the `Il2CppCodeRegistration` structure in the ELF binary.
    ///
    /// This high-level routine works as follows:
    ///
    /// 1. Finds all occurrences of `"mscorlib.dll\0"` in the ELF sections and converts file offsets to virtual addresses.
    /// 2. Finds first-level references (relocations) that point to any of these virtual addresses.
    /// 3. For each first-level reference, finds second-level references.
    /// 4. For each second-level reference, adjusts the address by subtracting an offset computed from the alphabetical
    ///    index of `"mscorlib.dll"` in the metadata images, then verifies candidates by checking that their `codeGenModulesCount`
    ///    matches the total number of images.
    ///
    /// # Arguments
    ///
    /// * `elf` - A reference to the ELF binary.
    /// * `metadata` - A reference to the global metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if any step fails to locate a valid `Il2CppCodeRegistration`.
    pub fn find_code_registration(
        elf: &Elf,
        metadata: &Metadata,
    ) -> Result<ReadOnly<Il2CppCodeRegistration>> {
        debug!("Finding Il2CppCodeRegistration...");
        // 1) Find file offsets of "mscorlib.dll\0" within the ELF data.
        const PATTERN: &[u8; 13] = b"mscorlib.dll\0";
        let mscorlib_file_offsets = elf.search_elf(PATTERN);

        // Convert each file offset to a virtual address.
        let mut mscorlib_vaddrs = Vec::new();
        for &file_off in &mscorlib_file_offsets {
            if let Some(program_header) = elf
                .inner
                .program_headers
                .iter()
                .find(|ph| ph.p_offset <= file_off && file_off < ph.p_offset + ph.p_filesz)
            {
                let offset_in_segment = file_off - program_header.p_offset;
                let va = program_header.p_vaddr + offset_in_segment;
                mscorlib_vaddrs.push(va);
            }
        }
        if mscorlib_vaddrs.is_empty() {
            bail!("No occurrences of 'mscorlib.dll' found in ELF");
        }

        // 2) Find references A: relocations that point to any of those virtual addresses.
        let mut mscorlib_refs: Vec<&u64> = Vec::new();
        for &mscorlib_va in &mscorlib_vaddrs {
            if let Some(relocs) = elf.relocations.get(&(mscorlib_va as i64)) {
                mscorlib_refs.extend(relocs);
            }
        }
        if mscorlib_refs.is_empty() {
            bail!("No references to 'mscorlib.dll' found");
        }

        // 3) For each reference A, find second-level references (B).
        let mut second_level_refs: Vec<u64> = Vec::new();
        for &ref_a in &mscorlib_refs {
            if let Some(relocs) = elf.relocations.get(&(*ref_a as i64)) {
                second_level_refs.extend(relocs);
            }
        }
        if second_level_refs.is_empty() {
            bail!("No second-level references found");
        }

        // 4) For each B, find references to `B - (mscorlib_index * POINTER_SIZE)`,
        //    which leads to the base of an Il2CppCodeRegistration struct.
        let mut image_names: Vec<String> = metadata
            .images
            .iter()
            .map(|img| metadata.get_string_by_index(img.nameIndex))
            .collect();

        // The image names are alphabetically sorted in the binary.
        image_names.sort();

        let mscorlib_idx = match image_names.binary_search(&"mscorlib.dll".to_string()) {
            Ok(idx) => idx as u64,
            Err(_) => bail!("mscorlib.dll not found in Metadata images"),
        };

        let images_ref_start = mscorlib_idx * POINTER_SIZE as u64;

        let mut possible_code_reg_bases: Vec<u64> = Vec::new();
        for &b_ref in &second_level_refs {
            let base_candidate = b_ref.wrapping_sub(images_ref_start);
            if let Some(relocs) = elf.relocations.get(&(base_candidate as i64)) {
                possible_code_reg_bases.extend(relocs);
            }
        }

        const CODEGEN_MODULES_OFFSET: usize = offset_of!(Il2CppCodeRegistration, codeGenModules);
        const CODE_REGISTRATION_SIZE: usize = size_of::<Il2CppCodeRegistration>();

        // Verify each candidate by reading an Il2CppCodeRegistration struct and checking its module count.
        let total_image_count = image_names.len() as u32;
        for &candidate_va in &possible_code_reg_bases {
            let struct_start_va = candidate_va.saturating_sub(CODEGEN_MODULES_OFFSET as u64);
            if let Some(bytes) = elf.read_bytes_at_va(struct_start_va, CODE_REGISTRATION_SIZE) {
                let code_reg = unsafe { *(bytes.as_ptr() as *const Il2CppCodeRegistration) };
                if code_reg.codeGenModulesCount == total_image_count {
                    debug!("Found Il2CppCodeRegistration at 0x{:x}", candidate_va);
                    return Ok(ReadOnly::new(code_reg));
                }
            }
        }

        Err(anyhow!("Could not find a valid Il2CppCodeRegistration"))
    }

    /// Locates the `Il2CppMetadataRegistration` structure within the ELF binary.
    ///
    /// The function searches for a byte pattern corresponding to the number of type definitions,
    /// then verifies potential candidates by checking adjacent fields and pointer validity.
    ///
    /// # Arguments
    ///
    /// * `elf` - A reference to the ELF binary.
    /// * `metadata` - A reference to the global metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if a valid `Il2CppMetadataRegistration` cannot be found.
    pub fn find_metadata_registration(
        elf: &Elf,
        metadata: &Metadata,
    ) -> Result<ReadOnly<Il2CppMetadataRegistration>> {
        debug!("Finding Il2CppMetadataRegistration...");
        let pattern = (metadata.type_definitions.len() as u64)
            .to_le_bytes()
            .to_vec();

        let field_count_file_offsets = elf.search_elf(&pattern);

        const TYPEDEF_SIZES_COUNT_OFFSET: usize =
            offset_of!(Il2CppMetadataRegistration, typeDefinitionsSizesCount);
        const METADATA_REGISTRATION_SIZE: usize = size_of::<Il2CppCodeRegistration>();

        let possible_metadata_regs = field_count_file_offsets
            .into_iter()
            .filter_map(|field_count_offset| {
                let type_count_offset = field_count_offset as usize + (POINTER_SIZE * 2);
                if type_count_offset > elf.data.len() - POINTER_SIZE {
                    return None;
                }
                if elf.data[type_count_offset..type_count_offset + pattern.len()] != pattern {
                    return None;
                }
                if let Some(candidate_va) = elf.file_offset_to_va(type_count_offset as u64) {
                    let struct_start_va =
                        candidate_va.saturating_sub(TYPEDEF_SIZES_COUNT_OFFSET as u64);
                    if let Some(bytes) =
                        elf.read_bytes_at_va(struct_start_va, METADATA_REGISTRATION_SIZE)
                    {
                        let metadata_reg =
                            unsafe { &*(bytes.as_ptr() as *const Il2CppMetadataRegistration) };
                        return Some(metadata_reg);
                    }
                }
                None
            })
            .collect::<Vec<_>>();

        match possible_metadata_regs.len() {
            0 => Err(anyhow!("Could not find a valid Il2CppMetadataRegistration")),
            1 => Ok(ReadOnly::new(*possible_metadata_regs[0])),
            _ => {
                for metadata_reg in possible_metadata_regs {
                    let type_defs_sizes_ptr_va = metadata_reg.typeDefinitionsSizes as u64;

                    if !elf.is_valid_pointer(type_defs_sizes_ptr_va) {
                        continue;
                    }

                    let type_defs_sizes_array = elf.read_pointer_array(
                        type_defs_sizes_ptr_va,
                        metadata_reg.typeDefinitionsSizesCount as usize,
                    );

                    if !type_defs_sizes_array
                        .iter()
                        .any(|&ptr| !elf.is_valid_pointer(ptr))
                    {
                        continue;
                    }

                    debug!(
                        "Found Il2CppMetadataRegistration at 0x{:x}",
                        metadata_reg as *const _ as u64
                    );
                    return Ok(ReadOnly::new(*metadata_reg));
                }
                Err(anyhow!("Could not find a valid Il2CppMetadataRegistration"))
            }
        }
    }

    /// Loads an instance of type `T` from the given virtual address in the ELF binary.
    ///
    /// This function reads the exact number of bytes required for type `T` and safely transmutes
    /// them into a reference of type `T`.
    ///
    /// # Arguments
    ///
    /// * `data_ptr` - The virtual address from which to load the data.
    ///
    /// # Returns
    ///
    /// Returns a reference to the instance of type `T` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the pointer is invalid or the data cannot be read.
    pub fn load_data_instance<T>(&'a self, data_ptr: u64) -> Result<&'a T> {
        Self::inner_load_data_instance(&self.elf, data_ptr)
    }

    /// Internal helper to load an instance of type `T` from the given virtual address in the ELF binary.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the virtual address is valid and points to an object of type `T`.
    ///
    /// # Arguments
    ///
    /// * `elf` - A reference to the ELF binary.
    /// * `data_ptr` - The virtual address to read the data from.
    ///
    /// # Returns
    ///
    /// Returns a reference to the loaded instance of type `T`.
    pub fn inner_load_data_instance<T>(elf: &Elf, data_ptr: u64) -> Result<&'a T> {
        if !elf.is_valid_pointer(data_ptr) {
            bail!("Invalid pointer");
        }

        let data_size = size_of::<T>();
        let data_bytes = elf
            .read_bytes_at_va(data_ptr, data_size)
            .ok_or(anyhow!("Failed to read data"))?;

        // SAFETY: The data slice is guaranteed to be the exact size of T before transmutation.
        let reference: &T = unsafe { &*(data_bytes.as_ptr() as *const T) };

        Ok(reference)
    }

    /// Loads an array of data instances of type `T` from a pointer to a pointer array in the ELF binary.
    ///
    /// This function reads a pointer array from the given virtual address and then loads each individual
    /// data instance by invoking `inner_load_data_instance`.
    ///
    /// # Arguments
    ///
    /// * `ptr` - A pointer to an array of pointers, each pointing to a data instance of type `T`.
    /// * `count` - The number of elements in the array.
    ///
    /// # Returns
    ///
    /// Returns a vector of references to instances of type `T`.
    pub fn load_data_array<T>(&'a self, ptr: *const *const T, count: usize) -> Result<Vec<&'a T>> {
        Self::inner_load_data_array(&self.elf, ptr, count)
    }

    /// Internal helper to load an array of data instances of type `T` from a pointer array in the ELF binary.
    ///
    /// # Arguments
    ///
    /// * `elf` - A reference to the ELF binary.
    /// * `ptr` - The virtual address of the pointer array.
    /// * `count` - The number of elements to read.
    ///
    /// # Returns
    ///
    /// Returns a vector of references to the loaded instances of type `T`.
    pub fn inner_load_data_array<T>(
        elf: &Elf,
        ptr: *const *const T,
        count: usize,
    ) -> Result<Vec<&'a T>> {
        if !elf.is_valid_pointer(ptr as u64) {
            bail!("Invalid pointer");
        }

        let data_ptr_array = elf.read_pointer_array(ptr as u64, count);

        let mut arr_refs = Vec::with_capacity(count);

        for &data_ptr in &data_ptr_array {
            arr_refs.push(Self::inner_load_data_instance::<T>(elf, data_ptr)?);
        }

        Ok(arr_refs)
    }

    /// Reads an encoded type enumeration from a data slice and optionally retrieves the associated type information.
    ///
    /// If the read byte indicates an `IL2CPP_TYPE_ENUM`, the function reads a compressed integer to determine
    /// the type index and returns the underlying type information.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of bytes containing the encoded type data.
    /// * `offset` - The offset within the data slice at which to start reading.
    ///
    /// # Returns
    ///
    /// Returns a tuple where the first element is the decoded type enumeration and the second element is an
    /// optional reference to additional type information.
    pub fn read_encoded_type_enum(
        &'a self,
        data: &[u8],
        offset: usize,
    ) -> (Il2CppTypeEnum, Option<&'a ReadOnly<&'a Il2CppType>>) {
        let ty = self.metadata.read_u8(data, offset) as Il2CppTypeEnum;
        if ty == IL2CPP_TYPE_ENUM {
            let ty_idx = self.metadata.read_compressed_i32(data, offset + 1);
            let ty = &self.types[ty_idx as usize];
            let ty_def = unsafe { self.metadata.type_definitions[ty.data.__klassIndex as usize] };
            (
                self.types[ty_def.elementTypeIndex as usize].type_(),
                Some(ty),
            )
        } else {
            (ty, None)
        }
    }

    /// Constructs a mapping of method pointers to their corresponding fully qualified method names.
    ///
    /// This function iterates over code generation modules and image metadata to combine information
    /// from the ELF binary and global metadata, thereby resolving method pointers to human-readable names.
    ///
    /// # Returns
    ///
    /// Returns an `IntMap` where the keys are method pointers and the values are formatted method names.
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the method extraction fails.
    pub fn methods(&'a self) -> Result<IntMap<u64, String>> {
        let code_reg = &self.code_registration;
        let code_gen_modules = self.load_data_array::<Il2CppCodeGenModule>(
            code_reg.codeGenModules,
            code_reg.codeGenModulesCount as usize,
        )?;

        let mut module_method_pointers = HashMap::with_capacity(code_gen_modules.len());
        for module in code_gen_modules {
            let module_name = self.elf.read_va_str(module.moduleName as u64, None)?;
            let pointers = self.elf.read_pointer_array(
                module.methodPointers as u64,
                module.methodPointerCount as usize,
            );
            module_method_pointers.insert(module_name, pointers);
        }

        let metadata = &self.metadata;

        let mut methods = IntMap::default();

        for image in &metadata.images {
            let image_name = metadata.get_string_by_index(image.nameIndex);
            let method_pointers = module_method_pointers.get(&image_name).ok_or(anyhow!(
                "Module method pointers should exist for each image"
            ))?;

            let type_end = image.typeStart as usize + image.typeCount as usize;
            for ty_idx in image.typeStart as usize..type_end {
                let ty_def = metadata.type_definitions[ty_idx];
                let ty = &self.types[ty_def.byvalTypeIndex as usize];
                let namespace = metadata.get_string_by_index(ty_def.namespaceIndex);
                let method_end = ty_def.methodStart as usize + ty_def.method_count as usize;
                for method_idx in ty_def.methodStart as usize..method_end {
                    let method = metadata.methods[method_idx];
                    let method_name = metadata.get_string_by_index(method.nameIndex);
                    // Adjust for zero-based index.
                    let pointer_idx = ((method.token & 0xFFFFFF) - 1) as usize;
                    if let Some(&method_ptr) = method_pointers.get(pointer_idx) {
                        if method_ptr > 0 {
                            let method_full_name = format!(
                                "{namespace}.{}$${method_name}",
                                ty.get_complex_type(self)?.get_name_str(true)?
                            );
                            methods.insert(method_ptr, method_full_name);
                        }
                    }
                }
            }
        }
        Ok(methods)
    }

    /// Retrieves type information corresponding to a given pointer.
    ///
    /// This function uses an internal mapping from type pointer addresses to indices to efficiently locate
    /// the associated IL2CPP type.
    ///
    /// # Arguments
    ///
    /// * `ptr` - The pointer value representing a type.
    ///
    /// # Returns
    ///
    /// Returns an optional reference to the type information if found.
    pub fn type_by_ptr(&self, ptr: u64) -> Option<&ReadOnly<&Il2CppType>> {
        let idx = *self.type_ptr_map.get(&ptr)?;
        let ty = self.types.get(idx)?;
        Some(ty)
    }
}
