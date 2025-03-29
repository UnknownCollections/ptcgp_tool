#![allow(dead_code)]

use crate::unity::generated::CIl2Cpp::{
    FieldIndex, Il2CppAssemblyDefinition, Il2CppCustomAttributeDataRange, Il2CppEventDefinition,
    Il2CppFieldDefaultValue, Il2CppFieldDefinition, Il2CppFieldMarshaledSize, Il2CppFieldRef,
    Il2CppGenericContainer, Il2CppGenericParameter, Il2CppGlobalMetadataHeader,
    Il2CppImageDefinition, Il2CppInterfaceOffsetPair, Il2CppMethodDefinition,
    Il2CppParameterDefaultValue, Il2CppParameterDefinition, Il2CppPropertyDefinition,
    Il2CppTypeDefinition, StringIndex,
};
use crate::unity::generated::SUPPORTED_GLOBAL_METADATA_VERSION;
use anyhow::{bail, Result};
use memchr::memchr;
use nohash_hasher::IntMap;
use paste::paste;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;

/// Reads a structure of type `T` from the given reader.
///
/// This function allocates uninitialized memory for `T` and then reads the exact number of
/// bytes from the reader to fill the structure. It is the caller’s responsibility to ensure
/// that the reader contains enough data.
///
/// # Errors
/// Returns an error if the reader cannot supply enough bytes.
fn read_struct<T: Copy, R: Read>(reader: &mut R) -> Result<T> {
    use std::mem::MaybeUninit;
    let mut data = MaybeUninit::<T>::uninit();
    let data_slice =
        unsafe { std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u8, size_of::<T>()) };
    reader.read_exact(data_slice)?;
    Ok(unsafe { data.assume_init() })
}

/// Reads an array of elements of type `T` from the given reader starting at a specified offset.
///
/// The total number of bytes to read is provided by `count_bytes`. The function calculates the
/// number of elements based on the size of `T` and returns a vector of these elements.
///
/// # Errors
/// Returns an error if the number of bytes is not a multiple of the element size or if reading fails.
fn read_array<T: Default + Copy, R: Read + Seek>(
    reader: &mut R,
    offset: i32,
    count_bytes: i32,
) -> Result<Vec<T>> {
    reader.seek(SeekFrom::Start(offset as u64))?;

    let elem_size = size_of::<T>() as i32;
    if elem_size == 0 {
        return Ok(Vec::new());
    }

    if count_bytes % elem_size != 0 {
        bail!(
            "Unused bytes detected: count_bytes ({}) is not a multiple of element size ({})",
            count_bytes,
            elem_size
        );
    }

    let num_elems = count_bytes / elem_size;
    let mut buffer = vec![T::default(); num_elems as usize];

    let buffer_slice = unsafe {
        std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, count_bytes as usize)
    };
    reader.read_exact(buffer_slice)?;

    Ok(buffer)
}

/// Macro to read an array from a file using a header's offset and size fields.
///
/// This macro simplifies calls to `read_array` by extracting the offset and size
/// from the provided header based on the given field name.
macro_rules! read_array {
    ($f:expr, $ty:ty, $h:expr, $name:ident) => {{
        paste! {
            read_array::<$ty, _>(
                &mut $f,
                $h.[<$name Offset>],
                $h.[<$name Size>],
            )?
        }
    }};
}

/// Represents the global metadata extracted from a Unity IL2CPP metadata file.
///
/// This structure contains various arrays and mappings of IL2CPP definitions and related data.
pub struct Metadata {
    /// The global metadata header providing offsets and sizes for subsequent data segments.
    pub header: Il2CppGlobalMetadataHeader,

    /// Raw data representing string literals.
    pub string_literal: Vec<u8>,
    /// Raw data representing the content of string literals.
    pub string_literal_data: Vec<u8>,
    /// Raw data for all strings used in the metadata.
    pub string_data: Vec<u8>,
    /// Cached mapping from string indices to their corresponding decoded strings.
    pub cached_strings: IntMap<StringIndex, String>,

    /// Array of event definitions.
    pub events: Vec<Il2CppEventDefinition>,
    /// Array of property definitions.
    pub properties: Vec<Il2CppPropertyDefinition>,
    /// Array of method definitions.
    pub methods: Vec<Il2CppMethodDefinition>,
    /// Array of default values for method parameters.
    pub parameter_default_values: Vec<Il2CppParameterDefaultValue>,
    /// Array of default values for fields.
    pub field_default_values: Vec<Il2CppFieldDefaultValue>,
    /// Mapping from field indices to their default value definitions.
    pub field_default_values_map: IntMap<FieldIndex, Il2CppFieldDefaultValue>,
    /// Raw data for default values of both fields and parameters.
    pub field_and_parameter_default_value_data: Vec<u8>,
    /// Array of marshaled sizes for fields.
    pub field_marshaled_sizes: Vec<Il2CppFieldMarshaledSize>,
    /// Array of parameter definitions.
    pub parameters: Vec<Il2CppParameterDefinition>,
    /// Array of field definitions.
    pub fields: Vec<Il2CppFieldDefinition>,
    /// Array of generic parameter definitions.
    pub generic_parameters: Vec<Il2CppGenericParameter>,
    /// Array of constraints for generic parameters.
    pub generic_parameter_constraints: Vec<i32>,
    /// Array of generic container definitions.
    pub generic_containers: Vec<Il2CppGenericContainer>,
    /// Array of nested type indices.
    pub nested_types: Vec<i32>,
    /// Array of interface indices.
    pub interfaces: Vec<i32>,
    /// Array of virtual table method indices.
    pub vtable_methods: Vec<u32>,
    /// Array of interface offset pairs.
    pub interface_offsets: Vec<Il2CppInterfaceOffsetPair>,
    /// Array of type definitions.
    pub type_definitions: Vec<Il2CppTypeDefinition>,
    /// Array of image definitions.
    pub images: Vec<Il2CppImageDefinition>,
    /// Array of assembly definitions.
    pub assemblies: Vec<Il2CppAssemblyDefinition>,
    /// Array of field references.
    pub field_refs: Vec<Il2CppFieldRef>,
    /// Array of referenced assembly indices.
    pub referenced_assemblies: Vec<i32>,
    /// Raw attribute data.
    pub attribute_data: Vec<u8>,
    /// Array of ranges defining custom attribute data.
    pub attribute_data_range: Vec<Il2CppCustomAttributeDataRange>,
    /// Raw data for unresolved indirect call parameter types.
    pub unresolved_indirect_call_parameter_types: Vec<u8>,
    /// Raw data for unresolved indirect call parameter ranges.
    pub unresolved_indirect_call_parameter_ranges: Vec<u8>,
    /// Raw data for Windows Runtime type names.
    pub windows_runtime_type_names: Vec<u8>,
    /// Raw data for Windows Runtime strings.
    pub windows_runtime_strings: Vec<u8>,
    /// Array of exported type definition indices.
    pub exported_type_definitions: Vec<i32>,
}

const GLOBAL_METADATA_MAGIC: i32 = -89056337;

impl Metadata {
    /// Loads the metadata from a reader by parsing its header and subsequent data segments.
    ///
    /// The function first reads the metadata header, performs sanity checks,
    /// and then reads each data segment based on offsets and sizes provided in the header.
    ///
    /// # Errors
    /// Returns an error if the header is invalid, the version is unsupported, or any read operation fails.
    pub fn load_from_reader<R: Read + Seek>(mut f: R) -> Result<Self> {
        let header: Il2CppGlobalMetadataHeader = read_struct(&mut f)?;

        // Sanity checks to ensure the metadata header is valid.
        if header.sanity != GLOBAL_METADATA_MAGIC {
            bail!("File does not have a valid header");
        }
        if header.version != SUPPORTED_GLOBAL_METADATA_VERSION {
            bail!("Metadata is described by unsupported version");
        }

        let string_literal = read_array!(f, u8, header, stringLiteral);
        let string_literal_data = read_array!(f, u8, header, stringLiteralData);
        let string_data = read_array!(f, u8, header, string);
        let cached_strings = Metadata::extract_null_terminated_strings(&string_data);

        let events = read_array!(f, Il2CppEventDefinition, header, events);
        let properties = read_array!(f, Il2CppPropertyDefinition, header, properties);
        let methods = read_array!(f, Il2CppMethodDefinition, header, methods);
        let parameter_default_values = read_array!(
            f,
            Il2CppParameterDefaultValue,
            header,
            parameterDefaultValues
        );
        let field_default_values =
            read_array!(f, Il2CppFieldDefaultValue, header, fieldDefaultValues);
        let field_default_values_map = IntMap::from_iter(
            field_default_values
                .iter()
                .map(|fdv| (fdv.fieldIndex, *fdv)),
        );
        let field_and_parameter_default_value_data =
            read_array!(f, u8, header, fieldAndParameterDefaultValueData);
        let field_marshaled_sizes =
            read_array!(f, Il2CppFieldMarshaledSize, header, fieldMarshaledSizes);
        let parameters = read_array!(f, Il2CppParameterDefinition, header, parameters);
        let fields = read_array!(f, Il2CppFieldDefinition, header, fields);
        let generic_parameters = read_array!(f, Il2CppGenericParameter, header, genericParameters);
        let generic_parameter_constraints =
            read_array!(f, i32, header, genericParameterConstraints);
        let generic_containers = read_array!(f, Il2CppGenericContainer, header, genericContainers);
        let nested_types = read_array!(f, i32, header, nestedTypes);
        let interfaces = read_array!(f, i32, header, interfaces);
        let vtable_methods = read_array!(f, u32, header, vtableMethods);
        let interface_offsets = read_array!(f, Il2CppInterfaceOffsetPair, header, interfaceOffsets);
        let type_definitions = read_array!(f, Il2CppTypeDefinition, header, typeDefinitions);
        let images = read_array!(f, Il2CppImageDefinition, header, images);
        let assemblies = read_array!(f, Il2CppAssemblyDefinition, header, assemblies);
        let field_refs = read_array!(f, Il2CppFieldRef, header, fieldRefs);
        let referenced_assemblies = read_array!(f, i32, header, referencedAssemblies);
        let attribute_data = read_array!(f, u8, header, attributeData);
        let attribute_data_range = read_array!(
            f,
            Il2CppCustomAttributeDataRange,
            header,
            attributeDataRange
        );
        let unresolved_indirect_call_parameter_types =
            read_array!(f, u8, header, unresolvedIndirectCallParameterTypes);
        let unresolved_indirect_call_parameter_ranges =
            read_array!(f, u8, header, unresolvedIndirectCallParameterRanges);
        let windows_runtime_type_names = read_array!(f, u8, header, windowsRuntimeTypeNames);
        let windows_runtime_strings = read_array!(f, u8, header, windowsRuntimeStrings);
        let exported_type_definitions = read_array!(f, i32, header, exportedTypeDefinitions);

        Ok(Self {
            header,
            string_literal,
            string_literal_data,
            string_data,
            cached_strings,
            events,
            properties,
            methods,
            parameter_default_values,
            field_default_values,
            field_default_values_map,
            field_and_parameter_default_value_data,
            field_marshaled_sizes,
            parameters,
            fields,
            generic_parameters,
            generic_parameter_constraints,
            generic_containers,
            nested_types,
            interfaces,
            vtable_methods,
            interface_offsets,
            type_definitions,
            images,
            assemblies,
            field_refs,
            referenced_assemblies,
            attribute_data,
            attribute_data_range,
            unresolved_indirect_call_parameter_types,
            unresolved_indirect_call_parameter_ranges,
            windows_runtime_type_names,
            windows_runtime_strings,
            exported_type_definitions,
        })
    }

    /// Retrieves a string from the metadata by its index.
    ///
    /// If the string is already cached, it returns the cached version.
    /// Otherwise, it decodes the string from the raw `string_data` using a null terminator.
    pub fn get_string_by_index(&self, index: StringIndex) -> String {
        if self.cached_strings.contains_key(&index) {
            self.cached_strings[&index].clone()
        } else {
            self.string_data[index as usize..]
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as char)
                .collect()
        }
    }

    /// Reads an unsigned 8-bit integer from the given data slice at the specified offset.
    pub fn read_u8(&self, data: &[u8], offset: usize) -> u8 {
        data[offset]
    }

    /// Reads a signed 8-bit integer from the given data slice at the specified offset.
    pub fn read_i8(&self, data: &[u8], offset: usize) -> i8 {
        data[offset] as i8
    }

    /// Reads an unsigned 16-bit integer from the given data slice at the specified offset.
    pub fn read_u16(&self, data: &[u8], offset: usize) -> u16 {
        let bytes: [u8; 2] = data[offset..offset + 2].try_into().unwrap();
        u16::from_le_bytes(bytes)
    }

    /// Reads a signed 16-bit integer from the given data slice at the specified offset.
    pub fn read_i16(&self, data: &[u8], offset: usize) -> i16 {
        self.read_u16(data, offset) as i16
    }

    /// Reads an unsigned 32-bit integer from the given data slice at the specified offset.
    pub fn read_u32(&self, data: &[u8], offset: usize) -> u32 {
        let bytes: [u8; 4] = data[offset..offset + 4].try_into().unwrap();
        u32::from_le_bytes(bytes)
    }

    /// Reads a signed 32-bit integer from the given data slice at the specified offset.
    pub fn read_i32(&self, data: &[u8], offset: usize) -> i32 {
        self.read_u32(data, offset) as i32
    }

    /// Reads an unsigned 64-bit integer from the given data slice at the specified offset.
    pub fn read_u64(&self, data: &[u8], offset: usize) -> u64 {
        let bytes: [u8; 8] = data[offset..offset + 8].try_into().unwrap();
        u64::from_le_bytes(bytes)
    }

    /// Reads a signed 64-bit integer from the given data slice at the specified offset.
    pub fn read_i64(&self, data: &[u8], offset: usize) -> i64 {
        self.read_u64(data, offset) as i64
    }

    /// Reads a 32-bit floating point number from the given data slice at the specified offset.
    pub fn read_f32(&self, data: &[u8], offset: usize) -> f32 {
        f32::from_bits(self.read_u32(data, offset))
    }

    /// Reads a 64-bit floating point number from the given data slice at the specified offset.
    pub fn read_f64(&self, data: &[u8], offset: usize) -> f64 {
        f64::from_bits(self.read_u64(data, offset))
    }

    /// Reads a compressed unsigned 32-bit integer from the given data slice starting at the specified offset.
    ///
    /// The compression scheme uses the first byte to determine the number of additional bytes to read.
    pub fn read_compressed_u32(&self, data: &[u8], offset: usize) -> u32 {
        let mut offset = offset;
        // First byte determines how many bytes to read.
        let first = data[offset];
        offset += 1;

        match first {
            // 1 byte
            0x00..=0x7F => first as u32,

            // 2 bytes: pattern 10xx xxxx (0x80..=0xBF)
            0x80..=0xBF => {
                debug_assert!(offset < data.len(), "Not enough data for 2-byte read");
                ((first as u32 & 0x7F) << 8) | (data[offset] as u32)
            }

            // 4 bytes: pattern 110x xxxx (0xC0..=0xDF)
            0xC0..=0xDF => {
                debug_assert!(offset + 2 < data.len(), "Not enough data for 4-byte read");
                let b1 = data[offset] as u32;
                let b2 = data[offset + 1] as u32;
                let b3 = data[offset + 2] as u32;

                ((first as u32 & 0x3F) << 24) | (b1 << 16) | (b2 << 8) | b3
            }

            // 5 bytes: 0xF0 indicates full 4 bytes in the next 4 bytes.
            0xF0 => {
                debug_assert!(offset + 3 < data.len(), "Not enough data for 5-byte read");
                let b1 = data[offset] as u32;
                let b2 = data[offset + 1] as u32;
                let b3 = data[offset + 2] as u32;
                let b4 = data[offset + 3] as u32;

                (b1 << 24) | (b2 << 16) | (b3 << 8) | b4
            }

            // 0xFE represents u32::MAX - 1.
            0xFE => u32::MAX - 1,

            // 0xFF represents u32::MAX.
            0xFF => u32::MAX,

            // Anything else is invalid.
            _ => panic!("Invalid compressed integer format: byte = 0x{:02X}", first),
        }
    }

    /// Reads a compressed signed 32-bit integer from the given data slice starting at the specified offset.
    ///
    /// Decodes the compressed format and adjusts the value to represent a signed integer.
    pub fn read_compressed_i32(&self, data: &[u8], offset: usize) -> i32 {
        let encoded = self.read_compressed_u32(data, offset);
        if encoded == u32::MAX {
            i32::MIN
        } else {
            ((encoded >> 1) as i32) ^ -((encoded & 1) as i32)
        }
    }

    /// Extracts null-terminated strings from the provided byte slice.
    ///
    /// Returns a mapping from the starting index (in the data slice) to the corresponding decoded string.
    fn extract_null_terminated_strings(data: &[u8]) -> IntMap<StringIndex, String> {
        let mut strings = IntMap::default();
        let mut pos = 0;

        while pos < data.len() {
            // Use memchr to quickly locate the next null terminator.
            if let Some(null_offset) = memchr(0, &data[pos..]) {
                let end = pos + null_offset;
                // Convert the slice to a String, handling invalid UTF-8 gracefully.
                let s = String::from_utf8_lossy(&data[pos..end]).into_owned();
                strings.insert(pos as StringIndex, s);
                // Move past the null terminator.
                pos = end + 1;
            } else {
                break;
            }
        }

        strings
    }
}
