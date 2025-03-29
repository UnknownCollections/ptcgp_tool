use crate::unity::blob_value::{BlobValue, BlobValueData};
use crate::unity::complex_type::{ComplexType, ComplexTypeArgs, ComplexTypeNamespace};
use crate::unity::generated::CIl2Cpp::{Il2CppType, Il2CppTypeDefinition};
use crate::unity::il2cpp::Il2Cpp;
use crate::utils::read_only::ReadOnly;
use anyhow::{anyhow, Result};
use phf::phf_map;

/// Maps IL2CPP type enum values to their corresponding string representations.
static TYPE_MAP: phf::Map<i32, &'static str> = phf_map! {
    1i32 => "void",
    2i32 => "bool",
    3i32 => "char",
    4i32 => "sbyte",
    5i32 => "byte",
    6i32 => "short",
    7i32 => "ushort",
    8i32 => "int",
    9i32 => "uint",
    10i32 => "long",
    11i32 => "ulong",
    12i32 => "float",
    13i32 => "double",
    14i32 => "string",
    22i32 => "TypedReference",
    24i32 => "IntPtr",
    25i32 => "UIntPtr",
    28i32 => "object",
};

impl<'a> ReadOnly<&'a Il2CppType> {
    /// Retrieves the type definition associated with this `Il2CppType`, if applicable.
    ///
    /// For class or value types, the type definition is directly retrieved. For generic instances,
    /// the type definition is extracted from the generic class.
    ///
    /// # Arguments
    ///
    /// * `il2cpp` - A reference to the `Il2Cpp` instance containing metadata.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(&Il2CppTypeDefinition))` if a type definition exists, or `Ok(None)` if not.
    pub fn get_type_def(
        &'a self,
        il2cpp: &'a Il2Cpp<'a>,
    ) -> Result<Option<&'a Il2CppTypeDefinition>> {
        use crate::unity::generated::CIl2Cpp::*;
        let ty_def = match self.type_() {
            IL2CPP_TYPE_CLASS | IL2CPP_TYPE_VALUETYPE => unsafe {
                &il2cpp.metadata.type_definitions[self.data.__klassIndex as usize]
            },
            IL2CPP_TYPE_GENERICINST => unsafe {
                // Load the generic class and its type instance to retrieve the underlying type definition.
                let generic_class = il2cpp
                    .load_data_instance::<Il2CppGenericClass>(self.data.generic_class as u64)?;
                let type_inst =
                    il2cpp.load_data_instance::<Il2CppType>(generic_class.type_ as u64)?;
                &il2cpp.metadata.type_definitions[type_inst.data.__klassIndex as usize]
            },
            _ => return Ok(None),
        };
        Ok(Some(ty_def))
    }

    /// Retrieves the declaring type of this `Il2CppType` if it is nested.
    ///
    /// Returns `None` if the type is not nested (i.e. `declaringTypeIndex` is negative) or if no declaring
    /// type is found.
    ///
    /// # Arguments
    ///
    /// * `il2cpp` - A reference to the `Il2Cpp` instance containing metadata.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(&ReadOnly<&Il2CppType>))` if a declaring type exists, or `Ok(None)` otherwise.
    pub fn get_declaring_type(
        &'a self,
        il2cpp: &'a Il2Cpp<'a>,
    ) -> Result<Option<&'a ReadOnly<&'a Il2CppType>>> {
        let res = if let Some(ty_def) = self.get_type_def(il2cpp)? {
            if ty_def.declaringTypeIndex < 0 {
                None
            } else {
                Some(&il2cpp.types[ty_def.declaringTypeIndex as usize])
            }
        } else {
            None
        };
        Ok(res)
    }

    /// Constructs the chain of declaring types starting from this `Il2CppType`.
    ///
    /// The chain includes the current type and all its declaring (outer) types in order.
    ///
    /// # Arguments
    ///
    /// * `il2cpp` - A reference to the `Il2Cpp` instance containing metadata.
    ///
    /// # Returns
    ///
    /// * A vector containing references to the `Il2CppType`s that form the declaring chain.
    pub fn get_declaring_chain(
        &'a self,
        il2cpp: &'a Il2Cpp<'a>,
    ) -> Result<Vec<&'a ReadOnly<&'a Il2CppType>>> {
        let mut chain = vec![self];
        let mut ty = self;
        while let Some(declaring_ty) = ty.get_declaring_type(il2cpp)? {
            chain.push(declaring_ty);
            ty = declaring_ty;
        }
        Ok(chain)
    }

    /// Converts this `Il2CppType` into a `ComplexType` representation.
    ///
    /// Handles various IL2CPP type kinds, including arrays, pointers, generic parameters, and generic instances.
    /// For basic types, a predefined mapping is used.
    ///
    /// # Arguments
    ///
    /// * `il2cpp` - A reference to the `Il2Cpp` instance containing metadata.
    ///
    /// # Returns
    ///
    /// * A `ComplexType` representing this IL2CPP type.
    pub fn get_complex_type(&'a self, il2cpp: &'a Il2Cpp<'a>) -> Result<ComplexType> {
        use crate::unity::generated::CIl2Cpp::*;
        let ty = self.type_();
        match ty {
            IL2CPP_TYPE_ARRAY => unsafe {
                // For arrays, retrieve the inner element type.
                let inner_ptr = (*self.data.array).etype as u64;
                let inner_ty = il2cpp
                    .type_by_ptr(inner_ptr)
                    .ok_or(anyhow!("Unknown array element type"))?;
                let inner = inner_ty.get_complex_type(il2cpp)?;
                Ok(ComplexType::Array(Box::new(inner)))
            },
            IL2CPP_TYPE_SZARRAY => unsafe {
                // Similar handling as IL2CPP_TYPE_ARRAY for single-dimension arrays.
                let inner_ptr = self.data.type_ as u64;
                let inner_ty = il2cpp
                    .type_by_ptr(inner_ptr)
                    .ok_or(anyhow!("Unknown array element type"))?;
                let inner = inner_ty.get_complex_type(il2cpp)?;
                Ok(ComplexType::Array(Box::new(inner)))
            },
            IL2CPP_TYPE_PTR => unsafe {
                // For pointers, retrieve the pointed-to type.
                let ptr = self.data.type_ as u64;
                let ptr_ty = il2cpp
                    .type_by_ptr(ptr)
                    .ok_or(anyhow!("Unknown pointer element type"))?;
                let pointee = ptr_ty.get_complex_type(il2cpp)?;
                Ok(ComplexType::Pointer(Box::new(pointee)))
            },
            IL2CPP_TYPE_VAR | IL2CPP_TYPE_MVAR => unsafe {
                // For generic parameters, construct a simple type using the parameter's name.
                let gen_param =
                    il2cpp.metadata.generic_parameters[self.data.__genericParameterIndex as usize];
                let name = il2cpp.metadata.get_string_by_index(gen_param.nameIndex);
                Ok(ComplexType::Simple {
                    module: None,
                    namespace: None,
                    name,
                    type_index: None,
                })
            },
            IL2CPP_TYPE_CLASS | IL2CPP_TYPE_VALUETYPE => unsafe {
                // For classes and value types, build a simple type from the type definition.
                let type_def = il2cpp.metadata.type_definitions[self.data.__klassIndex as usize];
                let simple = self.build_simple_from_typedef(il2cpp, &type_def)?;
                self.wrap_generic_container(il2cpp, &type_def, simple)
            },
            IL2CPP_TYPE_GENERICINST => unsafe {
                // Process a generic instance by loading the generic class and its type arguments.
                let generic_class = il2cpp
                    .load_data_instance::<Il2CppGenericClass>(self.data.generic_class as u64)?;
                let type_inst =
                    il2cpp.load_data_instance::<Il2CppType>(generic_class.type_ as u64)?;
                let type_def =
                    il2cpp.metadata.type_definitions[type_inst.data.__klassIndex as usize];
                let simple = self.build_simple_from_typedef(il2cpp, &type_def)?;
                let generic_inst = il2cpp.load_data_instance::<Il2CppGenericInst>(
                    generic_class.context.class_inst as u64,
                )?;
                let args = il2cpp.elf.read_pointer_array(
                    generic_inst.type_argv as u64,
                    generic_inst.type_argc as usize,
                );
                let generic_args: Result<Vec<_>> = args
                    .into_iter()
                    .map(|arg_ptr| {
                        let arg_ty = il2cpp
                            .type_by_ptr(arg_ptr)
                            .ok_or(anyhow!("Unknown generic arg"))?;
                        arg_ty.get_complex_type(il2cpp)
                    })
                    .collect();
                Ok(ComplexType::Generic {
                    base: Box::new(simple),
                    args: ComplexTypeArgs::new(generic_args?),
                })
            },
            _ => {
                // Use a predefined mapping for basic types. If not found, label as unknown.
                if let Some(ty_str) = TYPE_MAP.get(&(ty as i32)) {
                    Ok(ComplexType::Simple {
                        module: None,
                        namespace: None,
                        name: ty_str.to_string(),
                        type_index: None,
                    })
                } else {
                    Ok(ComplexType::Simple {
                        module: None,
                        namespace: None,
                        name: format!("unknown_{}", ty),
                        type_index: None,
                    })
                }
            }
        }
    }

    /// Constructs a simple `ComplexType` from the given type definition.
    ///
    /// Extracts the base name, namespace, and module information from the type definition.
    /// For nested types, it uses the declaring chain to determine the module name.
    ///
    /// # Safety
    ///
    /// This function is marked unsafe because it dereferences raw pointers from IL2CPP metadata.
    ///
    /// # Arguments
    ///
    /// * `il2cpp` - A reference to the `Il2Cpp` instance containing metadata.
    /// * `type_def` - A reference to the `Il2CppTypeDefinition` from which to build the `ComplexType`.
    ///
    /// # Returns
    ///
    /// * A simple `ComplexType` representing the type defined by `type_def`.
    unsafe fn build_simple_from_typedef(
        &'a self,
        il2cpp: &'a Il2Cpp<'a>,
        type_def: &Il2CppTypeDefinition,
    ) -> Result<ComplexType> {
        let raw_name = il2cpp.metadata.get_string_by_index(type_def.nameIndex);
        let ns = il2cpp.metadata.get_string_by_index(type_def.namespaceIndex);
        let module_name = if !ns.is_empty() || type_def.declaringTypeIndex == -1 {
            Some(ns)
        } else if let Some(outermost) = self.get_declaring_chain(il2cpp)?.last() {
            let module_name = outermost.get_complex_type(il2cpp)?;
            match module_name {
                ComplexType::Simple {
                    module: Some(module),
                    ..
                } => Some(module),
                _ => None,
            }
        } else {
            None
        };

        let base_name = raw_name.split('`').next().unwrap_or(&raw_name).to_string();

        if type_def.declaringTypeIndex != -1 {
            let declaring_type = &il2cpp.types[type_def.declaringTypeIndex as usize];
            let declaring_complex = declaring_type.get_complex_type(il2cpp)?;
            Ok(ComplexType::Simple {
                module: module_name,
                namespace: Some(ComplexTypeNamespace::Complex(Box::new(declaring_complex))),
                name: base_name,
                type_index: Some(type_def.byvalTypeIndex),
            })
        } else if let Some(pos) = base_name.rfind('.') {
            let (ns_part, name) = base_name.split_at(pos);
            // Skip the dot.
            Ok(ComplexType::Simple {
                module: module_name,
                namespace: Some(ComplexTypeNamespace::Simple(ns_part.to_string())),
                name: name[1..].to_string(),
                type_index: Some(type_def.byvalTypeIndex),
            })
        } else {
            Ok(ComplexType::Simple {
                module: module_name,
                namespace: None,
                name: base_name,
                type_index: Some(type_def.byvalTypeIndex),
            })
        }
    }

    /// Wraps the given simple `ComplexType` in a generic container if the type definition indicates one.
    ///
    /// If the type definition has a valid generic container, this function constructs a generic type with
    /// its base and generic arguments. Otherwise, it returns the original simple type.
    ///
    /// # Safety
    ///
    /// This function is marked unsafe because it dereferences raw pointers from IL2CPP metadata.
    ///
    /// # Arguments
    ///
    /// * `il2cpp` - A reference to the `Il2Cpp` instance containing metadata.
    /// * `type_def` - A reference to the `Il2CppTypeDefinition` that might have an associated generic container.
    /// * `simple` - The base `ComplexType` to potentially wrap in a generic container.
    ///
    /// # Returns
    ///
    /// * A `ComplexType` that is either the original simple type or a generic version of it.
    unsafe fn wrap_generic_container(
        &'a self,
        il2cpp: &'a Il2Cpp<'a>,
        type_def: &Il2CppTypeDefinition,
        simple: ComplexType,
    ) -> Result<ComplexType> {
        if type_def.genericContainerIndex >= 0 {
            let generic_container =
                &il2cpp.metadata.generic_containers[type_def.genericContainerIndex as usize];
            let generic_args: Vec<ComplexType> = (0..generic_container.type_argc)
                .map(|i| {
                    let idx = generic_container.genericParameterStart + i;
                    let param = il2cpp.metadata.generic_parameters[idx as usize];
                    let name = il2cpp.metadata.get_string_by_index(param.nameIndex);
                    ComplexType::Simple {
                        module: None,
                        namespace: None,
                        name,
                        type_index: None,
                    }
                })
                .collect();
            Ok(ComplexType::Generic {
                base: Box::new(simple),
                args: ComplexTypeArgs::new(generic_args),
            })
        } else {
            Ok(simple)
        }
    }

    /// Reads and interprets a value of this `Il2CppType` from a byte slice.
    ///
    /// Based on the IL2CPP type enumeration, this function parses the raw data at the specified offset and
    /// returns a `BlobValue` encapsulating the interpreted value.
    ///
    /// Supports various primitive types, strings, arrays, and type indices.
    ///
    /// # Arguments
    ///
    /// * `il2cpp` - A reference to the `Il2Cpp` instance containing metadata.
    /// * `data` - The byte slice from which to read the value.
    /// * `offset` - The offset in the byte slice at which the value begins.
    ///
    /// # Returns
    ///
    /// * A `BlobValue` representing the parsed value.
    pub fn get_value(
        &self,
        il2cpp: &'a Il2Cpp<'a>,
        data: &[u8],
        offset: usize,
    ) -> Result<BlobValue> {
        use crate::unity::generated::CIl2Cpp::*;

        let ty = self.type_();

        match ty {
            IL2CPP_TYPE_BOOLEAN => {
                // Booleans are stored as a single byte (non-zero is true).
                let b = il2cpp.metadata.read_u8(data, offset) != 0;
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::Boolean(b),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_U1 => {
                let b = il2cpp.metadata.read_u8(data, offset);
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::U1(b),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_I1 => {
                let b = il2cpp.metadata.read_i8(data, offset);
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::I1(b),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_CHAR => {
                let num = il2cpp.metadata.read_u16(data, offset);
                let c = std::char::from_u32(num as u32)
                    .ok_or_else(|| anyhow!("Invalid char value: {}", num))?;
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::Char(c),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_U2 => {
                let v = il2cpp.metadata.read_u16(data, offset);
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::U2(v),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_I2 => {
                let v = il2cpp.metadata.read_i16(data, offset);
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::I2(v),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_U4 => {
                let v = il2cpp.metadata.read_compressed_u32(data, offset);
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::U4(v),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_I4 => {
                let v = il2cpp.metadata.read_compressed_i32(data, offset);
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::I4(v),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_U8 => {
                let v = il2cpp.metadata.read_u64(data, offset);
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::U8(v),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_I8 => {
                let v = il2cpp.metadata.read_i64(data, offset);
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::I8(v),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_R4 => {
                let v = il2cpp.metadata.read_f32(data, offset);
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::R4(v),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_R8 => {
                let v = il2cpp.metadata.read_f64(data, offset);
                Ok(BlobValue {
                    il2cpp_type_enum: ty,
                    value: BlobValueData::R8(v),
                    enum_type: None,
                })
            }
            IL2CPP_TYPE_STRING => {
                // Read the length of the string. A length of -1 indicates an empty string.
                let length = il2cpp.metadata.read_compressed_i32(data, offset);
                if length == -1 {
                    Ok(BlobValue {
                        il2cpp_type_enum: ty,
                        value: BlobValueData::String(String::new()),
                        enum_type: None,
                    })
                } else {
                    let s = String::from_utf8(data[offset..offset + length as usize].to_vec())?;
                    Ok(BlobValue {
                        il2cpp_type_enum: ty,
                        value: BlobValueData::String(s),
                        enum_type: None,
                    })
                }
            }
            IL2CPP_TYPE_SZARRAY => {
                // Read the length of the single-dimension array.
                let array_len = il2cpp.metadata.read_compressed_i32(data, offset);
                if array_len == -1 {
                    Ok(BlobValue {
                        il2cpp_type_enum: ty,
                        value: BlobValueData::Array(Vec::new()),
                        enum_type: None,
                    })
                } else {
                    let mut array = Vec::with_capacity(array_len as usize);
                    // Read the element type and an optional enum type for the array elements.
                    let (array_element_type, mut elem_enum_type) =
                        il2cpp.read_encoded_type_enum(data, offset);
                    let array_elements_are_different = il2cpp.metadata.read_u8(data, offset);
                    for _ in 0..array_len {
                        let element_type = if array_elements_are_different == 1 {
                            // If elements have different types, read the type for each element.
                            let (et, et_enum) = il2cpp.read_encoded_type_enum(data, offset);
                            elem_enum_type = et_enum;
                            et
                        } else {
                            array_element_type
                        };
                        let mut element_value = self.get_value(il2cpp, data, offset)?;
                        element_value.il2cpp_type_enum = element_type;
                        element_value.enum_type = elem_enum_type.map(|t| **(*t));
                        array.push(element_value);
                    }
                    Ok(BlobValue {
                        il2cpp_type_enum: ty,
                        value: BlobValueData::Array(array),
                        enum_type: None,
                    })
                }
            }
            IL2CPP_TYPE_IL2CPP_TYPE_INDEX => {
                let type_index = il2cpp.metadata.read_compressed_i32(data, offset);
                if type_index == -1 {
                    Ok(BlobValue {
                        il2cpp_type_enum: ty,
                        value: BlobValueData::TypeIndex(None),
                        enum_type: None,
                    })
                } else {
                    let type_obj = il2cpp.types.get(type_index as usize).map(|t| *(**t));
                    Ok(BlobValue {
                        il2cpp_type_enum: ty,
                        value: BlobValueData::TypeIndex(type_obj),
                        enum_type: None,
                    })
                }
            }
            _ => Err(anyhow!("Unsupported type in get_value: {:?}", ty)),
        }
    }
}
