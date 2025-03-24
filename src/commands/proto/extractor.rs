use crate::proto::field::{ProtoCardinality, ProtoField};
use crate::proto::map::ProtoMapField;
use crate::proto::message::ProtoMessage;
use crate::proto::one_of::ProtoOneOf;
use crate::proto::package::ProtoPackage;
use crate::proto::proto_enum::ProtoEnum;
use crate::proto::schema::{ProtoGenSchema, ProtoSchema};
use crate::proto::service::{ProtoService, ProtoServiceMethod};
use crate::proto::ProtoType;
use crate::unity::complex_type::ComplexType;
use crate::unity::generated::CIl2Cpp::{Il2CppImageDefinition, Il2CppTypeDefinition, TypeIndex};
use crate::unity::il2cpp::Il2Cpp;
use anyhow::{anyhow, bail, Result};
use hashbrown::HashMap;
use log::debug;
use phf::phf_map;
use std::cell::RefCell;
use std::rc::Rc;

/// Generates a protobuf schema by scanning IL2CPP metadata and mapping .NET types to proto definitions.
///
/// This function loads IL2CPP metadata and iterates through its images, processing each image for services,
/// enums, and message types. Nested types and oneof cases are collected and merged appropriately into the
/// resulting schema.
///
/// # Arguments
/// * `il2cpp` - `Il2Cpp` instance
///
/// # Returns
/// * `ProtoGenSchema` on success or an error if schema generation fails.
pub fn generate_proto_schema(il2cpp: Il2Cpp) -> Result<ProtoGenSchema> {
    let mut schema = ProtoSchema::new();

    // Maps for collecting nested types and oneof case enums.
    let mut nested_types_map: HashMap<TypeIndex, Vec<ProtoType>> = HashMap::new();
    let mut oneof_cases: HashMap<TypeIndex, Vec<ProtoEnum>> = HashMap::new();

    // Process each image in the IL2CPP metadata.
    debug!(progress = 0, max = il2cpp.metadata.images.len(); "");
    for game_image in &il2cpp.metadata.images {
        process_image(
            game_image,
            &il2cpp,
            &mut schema,
            &mut nested_types_map,
            &mut oneof_cases,
        )?;
        debug!(progress_tick = 1; "");
    }

    // Resolve and merge nested types.
    process_nested_types(&il2cpp, &mut nested_types_map)?;
    integrate_nested_types_into_packages(&mut schema, &mut nested_types_map);

    schema.seal();
    debug!("Build generated proto schema...");
    schema.build()
}

/// A mapping from .NET type names to their corresponding protobuf type names.
///
/// This mapping is used to translate known .NET primitive types to their equivalent proto types.
static NET_TO_PROTO: phf::Map<&'static str, &'static str> = phf_map! {
    "int" => "int32",
    "Int32" => "int32",
    "long" => "int64",
    "Int64" => "int64",
    "ulong" => "fixed64",
    "UInt64" => "fixed64",
    "uint" => "fixed32",
    "UInt32" => "fixed32",
    "Single" => "float",
    "Boolean" => "bool",
    "Double" => "double",
    "String" => "string",
    "ByteString" => "bytes",
};

/// Processes a single IL2CPP image by iterating through its type definitions and generating corresponding
/// proto definitions.
///
/// It examines each type definition in the image to determine if it represents a service, an enum, or a message.
/// The function then delegates processing to the appropriate helper function.
///
/// # Arguments
/// * `game_image` - Reference to the IL2CPP image definition containing type information.
/// * `il2cpp` - Reference to the IL2CPP context holding metadata and type definitions.
/// * `schema` - Mutable reference to the protobuf schema being built.
/// * `nested_types_map` - Mutable reference to a map collecting nested proto types.
/// * `oneof_cases` - Mutable reference to a map collecting oneof enum cases.
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Returns Ok(()) if processing is successful; otherwise, an error.
fn process_image<'a>(
    game_image: &Il2CppImageDefinition,
    il2cpp: &'a Il2Cpp<'a>,
    schema: &mut ProtoSchema,
    nested_types_map: &mut HashMap<TypeIndex, Vec<ProtoType>>,
    oneof_cases: &mut HashMap<TypeIndex, Vec<ProtoEnum>>,
) -> Result<()> {
    let image_name = il2cpp.metadata.get_string_by_index(game_image.nameIndex);
    debug!("Processing IL2CPP image: {image_name}");

    let type_start = game_image.typeStart as usize;
    let type_end = type_start + game_image.typeCount as usize;
    let type_defs = &il2cpp.metadata.type_definitions[type_start..type_end];

    for ty_def in type_defs {
        let namespace = il2cpp.metadata.get_string_by_index(ty_def.namespaceIndex);

        let package = schema.get(namespace);

        // Determine if this type represents a service, enum, or message.
        if ty_def.has_field(il2cpp, "__ServiceName", "string") {
            process_service(ty_def, il2cpp, package)?;
        } else if ty_def.is_enum_type() {
            process_enum(ty_def, il2cpp, package, nested_types_map, oneof_cases)?;
        } else if ty_def.has_field(il2cpp, "_parser", "MessageParser") {
            process_message(ty_def, il2cpp, package, nested_types_map, oneof_cases)?;
        }
    }
    Ok(())
}

/// Processes a gRPC service type by identifying its client type and extracting its RPC methods.
///
/// This function verifies that the service type contains the expected fields and locates the corresponding
/// client type definition to inspect available RPC methods. It then creates and adds proto service definitions
/// to the given package.
///
/// # Arguments
/// * `ty_def` - Reference to the IL2CPP type definition representing the service.
/// * `il2cpp` - Reference to the IL2CPP context.
/// * `package` - Mutable reference to the proto package to which the service will be added.
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Ok if processing succeeds; otherwise, an error.
fn process_service<'a>(
    ty_def: &Il2CppTypeDefinition,
    il2cpp: &'a Il2Cpp<'a>,
    package: &mut ProtoPackage,
) -> Result<()> {
    let metadata = &il2cpp.metadata;
    let service_name = metadata.get_string_by_index(ty_def.nameIndex);
    let expected_client_name = format!("{}Client", service_name);
    debug!("Processing GRPC proto service: {service_name}");

    // Find the client type definition matching the expected client name.
    let service_client_ty_def = metadata
        .type_definitions
        .iter()
        .find(|td| {
            td.declaringTypeIndex == ty_def.byvalTypeIndex
                && metadata.get_string_by_index(td.nameIndex) == expected_client_name
        })
        .ok_or_else(|| anyhow!("Could not find client type for service {}", service_name))?;

    let mut service = ProtoService::new(service_name.clone(), ty_def.byvalTypeIndex);

    for field_idx in ty_def.get_field_range() {
        let field = &metadata.fields[field_idx];
        let field_name = metadata.get_string_by_index(field.nameIndex);

        // Only process fields that indicate RPC methods.
        if !field_name.starts_with("__Method_") {
            continue;
        }

        let rpc_name = field_name.trim_start_matches("__Method_");

        let method_field_type = &il2cpp.types[field.typeIndex as usize];
        match method_field_type.get_complex_type(il2cpp)? {
            ComplexType::Generic { ref base, ref args } => {
                // Expect the base generic type to be "Method".
                if base.to_string() != "Method" {
                    bail!("Unexpected base type in service method: {}", base);
                }
                if args.args.len() < 2 {
                    bail!("Service method field does not have two type arguments");
                }

                let request_type = &args.args[0];
                let response_type = &args.args[1];

                let (client_streaming, server_streaming) =
                    get_rpc_streaming_info(service_client_ty_def, rpc_name, il2cpp)?;

                let rpc_method = ProtoServiceMethod::new(
                    rpc_name.to_string(),
                    request_type.get_root_namespace().map(String::from),
                    request_type.get_name_str(true)?,
                    request_type.get_type_index(),
                    response_type.get_root_namespace().map(String::from),
                    response_type.get_name_str(true)?,
                    response_type.get_type_index(),
                    client_streaming,
                    server_streaming,
                );

                service.add_method(rpc_method);
            }
            other => {
                bail!("Unexpected method field type: {:?}", other);
            }
        }
    }
    package.add_service(service);
    Ok(())
}

/// Determines the streaming configuration for an RPC method by inspecting the client's method signature.
///
/// It searches for a method in the client type that matches the RPC name and examines its return type to
/// determine if the method supports client streaming, server streaming, or both.
///
/// # Arguments
/// * `client_ty_def` - Reference to the client type definition.
/// * `rpc_name` - The name of the RPC method.
/// * `il2cpp` - Reference to the IL2CPP context.
///
/// # Returns
/// * `Result<(bool, bool), Box<dyn Error>>` - A tuple indicating (client_streaming, server_streaming).
///   If no matching method is found, both values default to false.
fn get_rpc_streaming_info<'a>(
    client_ty_def: &Il2CppTypeDefinition,
    rpc_name: &str,
    il2cpp: &'a Il2Cpp<'a>,
) -> Result<(bool, bool)> {
    let metadata = &il2cpp.metadata;
    let start = client_ty_def.methodStart as usize;
    let end = start + client_ty_def.method_count as usize;
    let methods = &metadata.methods[start..end];

    for method in methods {
        if metadata.get_string_by_index(method.nameIndex) == rpc_name {
            let return_type = &il2cpp.types[method.returnType as usize];
            return match return_type.get_complex_type(il2cpp)? {
                ComplexType::Generic { ref base, .. } => {
                    let base_name = base.to_string();
                    if base_name == "AsyncDuplexStreamingCall" {
                        Ok((true, true))
                    } else if base_name == "AsyncClientStreamingCall" {
                        Ok((true, false))
                    } else if base_name == "AsyncServerStreamingCall" {
                        Ok((false, true))
                    } else {
                        // For any other type, assume no streaming.
                        Ok((false, false))
                    }
                }
                _ => Ok((false, false)),
            };
        }
    }
    // If no matching method is found, default to non-streaming.
    Ok((false, false))
}

/// Processes an enum type, handling both standard enums and oneof case enums.
///
/// For oneof cases, the enum is collected separately to be integrated into its parent message later.
/// Non-oneof enums are added directly to the package or stored as nested types if declared within another type.
///
/// # Arguments
/// * `ty_def` - Reference to the IL2CPP type definition representing the enum.
/// * `il2cpp` - Reference to the IL2CPP context.
/// * `package` - Mutable reference to the proto package for top-level enums.
/// * `nested_types_map` - Mutable map for storing nested proto types.
/// * `oneof_cases` - Mutable map for storing oneof case enums.
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Ok if processing succeeds; otherwise, an error.
fn process_enum<'a>(
    ty_def: &Il2CppTypeDefinition,
    il2cpp: &'a Il2Cpp<'a>,
    package: &mut ProtoPackage,
    nested_types_map: &mut HashMap<TypeIndex, Vec<ProtoType>>,
    oneof_cases: &mut HashMap<TypeIndex, Vec<ProtoEnum>>,
) -> Result<()> {
    let enum_type = parse_enum_type(il2cpp, ty_def)?;
    debug!("Processing proto enum: {}", enum_type.name);
    if enum_type.name.ends_with("OneofCase") {
        oneof_cases
            .entry(ty_def.declaringTypeIndex)
            .or_default()
            .push(enum_type);
    } else if ty_def.declaringTypeIndex >= 0 {
        nested_types_map
            .entry(ty_def.declaringTypeIndex)
            .or_default()
            .push(ProtoType::Enum(enum_type));
    } else {
        package.add_enum(enum_type);
    }
    Ok(())
}

/// Processes a message type by scanning its fields and generating corresponding proto fields,
/// including handling oneof groups.
///
/// The function identifies fields using a specific naming pattern, retrieves their default numeric values,
/// and determines their types. For fields belonging to a oneof group, they are grouped accordingly.
/// Nested message types are either recorded for later processing or added directly to the package.
///
/// # Arguments
/// * `ty_def` - Reference to the IL2CPP type definition representing the message.
/// * `il2cpp` - Reference to the IL2CPP context.
/// * `package` - Mutable reference to the proto package for top-level messages.
/// * `nested_types_map` - Mutable map for storing nested proto types.
/// * `oneof_cases` - Mutable map for storing oneof case enums.
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Ok if processing succeeds; otherwise, an error.
fn process_message<'a>(
    ty_def: &Il2CppTypeDefinition,
    il2cpp: &'a Il2Cpp<'a>,
    package: &mut ProtoPackage,
    nested_types_map: &mut HashMap<TypeIndex, Vec<ProtoType>>,
    oneof_cases: &mut HashMap<TypeIndex, Vec<ProtoEnum>>,
) -> Result<()> {
    let message_name = il2cpp.metadata.get_string_by_index(ty_def.nameIndex);
    debug!("Processing proto message: {message_name}");
    let mut new_message = ProtoMessage::create(message_name, ty_def.byvalTypeIndex);

    // Build oneof mapping if oneof enums exist.
    let mut oneof_field_map: HashMap<String, Rc<RefCell<ProtoOneOf>>> = HashMap::new();
    let mut oneof_fields = Vec::new();
    if let Some(oneof_enums) = oneof_cases.remove(&ty_def.byvalTypeIndex) {
        for oneof_enum in oneof_enums {
            let oneof_name = oneof_enum
                .name
                .strip_suffix("OneofCase")
                .unwrap_or(&oneof_enum.name)
                .to_ascii_lowercase();
            let proto_field = Rc::new(RefCell::new(ProtoOneOf::create(oneof_name)));
            oneof_fields.push(proto_field.clone());
            // Map each variant name to its corresponding oneof group.
            for variant_name in oneof_enum.variants.keys() {
                oneof_field_map.insert(variant_name.clone(), proto_field.clone());
            }
        }
    }

    // Cache the method slice associated with this type.
    let methods_slice = {
        let start = ty_def.methodStart as usize;
        let end = start + ty_def.method_count as usize;
        &il2cpp.metadata.methods[start..end]
    };

    // Process each field that matches the expected naming pattern.
    for field_idx in ty_def.get_field_range() {
        let field = &il2cpp.metadata.fields[field_idx];
        let field_name = il2cpp.metadata.get_string_by_index(field.nameIndex);
        let proto_field_name = match field_name.strip_suffix("FieldNumber") {
            Some(n) => n,
            None => continue,
        };
        let proto_field_number = get_field_default_numeric_value(il2cpp, field_idx as i32)?;
        let getter_name = format!("get_{}", proto_field_name);

        // Find the getter method to determine the field's type.
        if let Some(method) = methods_slice
            .iter()
            .find(|m| il2cpp.metadata.get_string_by_index(m.nameIndex) == getter_name)
        {
            let return_type = &il2cpp.types[method.returnType as usize];
            match return_type.get_complex_type(il2cpp)? {
                ComplexType::Simple {
                    mut module,
                    namespace,
                    name,
                    mut type_index,
                } => {
                    let simple_name = if let Some(ref ns) = namespace {
                        format!("{}.{}", ns, name)
                    } else {
                        name
                    };
                    let simple_type_name = if let Some(proto_name) = NET_TO_PROTO.get(&simple_name)
                    {
                        module = None;
                        type_index = None;
                        proto_name.to_string()
                    } else {
                        simple_name
                    };
                    let field_obj = ProtoField::new(
                        module,
                        proto_field_name.to_string(),
                        simple_type_name,
                        type_index,
                        proto_field_number,
                        None,
                    );
                    if let Some(oneof_field) = oneof_field_map.get(proto_field_name) {
                        oneof_field.borrow_mut().add_field(field_obj);
                    } else {
                        new_message.add_field(field_obj);
                    }
                }
                ComplexType::Generic { base, args } => {
                    let base_name = base.to_string();
                    if base_name == "MapField" {
                        new_message.add_map_field(ProtoMapField::new(
                            args.args[0].to_string(),
                            args.args[0].get_type_index(),
                            args.args[1].to_string(),
                            args.args[1].get_type_index(),
                            proto_field_name.to_string(),
                            proto_field_number,
                        ));
                    } else {
                        let cardinality = match base_name.as_str() {
                            "Nullable" => Some(ProtoCardinality::Optional),
                            "RepeatedField" => Some(ProtoCardinality::Repeated),
                            _ => unimplemented!("Cardinality unsupported: {}<{:?}>", base, args),
                        };
                        let mut module_name = args.get_module_name();
                        let inner_type = args.get_name_str(true)?;
                        let mut field_type_index = args.args[0].get_type_index();
                        let type_name = if let Some(proto_name) = NET_TO_PROTO.get(&inner_type) {
                            module_name = None;
                            field_type_index = None;
                            proto_name.to_string()
                        } else {
                            inner_type
                        };
                        let field_obj = ProtoField::new(
                            module_name,
                            proto_field_name.to_string(),
                            type_name,
                            field_type_index,
                            proto_field_number,
                            cardinality,
                        );
                        if let Some(oneof_field) = oneof_field_map.get(proto_field_name) {
                            oneof_field.borrow_mut().add_field(field_obj);
                        } else {
                            new_message.add_field(field_obj);
                        }
                    }
                }
                ct => unimplemented!("Complex type unsupported: {:?}", ct),
            }
        }
    }

    // Add the assembled oneof groups to the message.
    for oneof_field in oneof_fields {
        new_message.add_oneof(oneof_field.borrow().to_owned());
    }

    // If the message is nested within another type, record it for later integration.
    if ty_def.declaringTypeIndex >= 0 {
        nested_types_map
            .entry(ty_def.declaringTypeIndex)
            .or_default()
            .push(ProtoType::Message(new_message));
    } else {
        package.add_message(new_message);
    }
    Ok(())
}

/// Processes nested types by resolving the declaring chain and merging nested messages into their parent types.
///
/// It walks the chain of declaring types for nested types and creates a nested message structure, ensuring that
/// nested enums and messages are appropriately merged.
///
/// # Arguments
/// * `il2cpp` - Reference to the IL2CPP context.
/// * `nested_types_map` - Mutable map that associates type indices with their nested proto types.
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Ok if successful; otherwise, an error.
fn process_nested_types<'a>(
    il2cpp: &'a Il2Cpp<'a>,
    nested_types_map: &mut HashMap<TypeIndex, Vec<ProtoType>>,
) -> Result<()> {
    debug!("Merge nested types...");
    let nested_type_indexes: Vec<TypeIndex> = nested_types_map.keys().copied().collect();
    for ty_idx in nested_type_indexes {
        let first_parent_ty = &il2cpp.types[ty_idx as usize];
        let mut ty_chain = first_parent_ty.get_declaring_chain(il2cpp)?;
        // A chain length of 1 indicates no nesting.
        if ty_chain.len() == 1 {
            continue;
        }
        // The deepest member of the chain will be the target to nest into.
        let new_target_ty_def = ty_chain.pop().unwrap().get_type_def(il2cpp)?.unwrap();
        let mut new_message: Option<ProtoMessage> = None;
        // Build the nested chain from outer types down to the deepest nested type.
        for ty in ty_chain {
            let ty_def = ty.get_type_def(il2cpp)?.unwrap();
            let ty_name = ty.get_complex_type(il2cpp)?;
            let mut ty_message =
                ProtoMessage::create(ty_name.get_name_str(false)?, ty_def.byvalTypeIndex);
            if let Some(base) = new_message.take() {
                ty_message.nested_messages.push(base);
            } else if let Some(nested_types) = nested_types_map.remove(&ty_idx) {
                for nested_type in nested_types {
                    match nested_type {
                        ProtoType::Enum(enum_type) => ty_message.nested_enums.push(enum_type),
                        ProtoType::Message(message) => ty_message.nested_messages.push(message),
                        _ => unreachable!(),
                    }
                }
            }
            new_message = Some(ty_message);
        }
        let new_message = new_message.unwrap();
        let nested_types = nested_types_map
            .entry(new_target_ty_def.byvalTypeIndex)
            .or_default();
        if let Some(ProtoType::Message(existing_msg)) =
            nested_types.iter_mut().find(|nested_type| {
                if let ProtoType::Message(msg) = nested_type {
                    msg.type_index == new_message.type_index
                } else {
                    false
                }
            })
        {
            existing_msg.merge(new_message);
        } else {
            nested_types.push(ProtoType::Message(new_message));
        }
    }
    Ok(())
}

/// Integrates any remaining nested types into their corresponding parent messages within the protobuf packages.
///
/// This function iterates through all packages and attaches nested enums and messages to the parent message
/// that corresponds to the type index.
///
/// # Arguments
/// * `schema` - Mutable reference to the protobuf schema containing packages.
/// * `nested_types_map` - Mutable map of nested proto types that have not yet been integrated.
fn integrate_nested_types_into_packages(
    schema: &mut ProtoSchema,
    nested_types_map: &mut HashMap<TypeIndex, Vec<ProtoType>>,
) {
    debug!("Integrate nested types into packages...");
    for namespace in schema.packages.values_mut() {
        for message in namespace.messages_mut() {
            if let Some(nested_types) = nested_types_map.remove(&message.type_index) {
                for nested_type in nested_types {
                    match nested_type {
                        ProtoType::Enum(e) => message.nested_enums.push(e),
                        ProtoType::Message(m) => message.nested_messages.push(m),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}

/// Retrieves the default numeric value for a given field using IL2CPP metadata.
///
/// It looks up the field index in the default values map and extracts its value by interpreting the associated type.
///
/// # Arguments
/// * `il2cpp` - Reference to the IL2CPP context.
/// * `field_index` - The index of the field for which the default value is to be retrieved.
///
/// # Returns
/// * `Result<i32, Box<dyn Error>>` - The default numeric value as an integer.
fn get_field_default_numeric_value<'a>(il2cpp: &'a Il2Cpp<'a>, field_index: i32) -> Result<i32> {
    let fdv = il2cpp
        .metadata
        .field_default_values_map
        .get(&field_index)
        .ok_or_else(|| {
            anyhow!(
                "No field default value found for field index {}",
                field_index
            )
        })?;
    let fdv_ty = &il2cpp.types[fdv.typeIndex as usize];
    let value = fdv_ty.get_value(
        il2cpp,
        &il2cpp.metadata.field_and_parameter_default_value_data,
        fdv.dataIndex as usize,
    )?;
    Ok(value.as_num()? as i32)
}

/// Parses an IL2CPP enum type into a corresponding `ProtoEnum`.
///
/// This function extracts the enum name and iterates through its fields (skipping the first "__value" field)
/// to populate the enum variants with their default numeric values.
///
/// # Arguments
/// * `il2cpp` - Reference to the IL2CPP context.
/// * `ty_def` - Reference to the IL2CPP type definition representing the enum.
///
/// # Returns
/// * `Result<ProtoEnum, Box<dyn Error>>` - The constructed ProtoEnum or an error if parsing fails.
fn parse_enum_type<'a>(il2cpp: &'a Il2Cpp<'a>, ty_def: &Il2CppTypeDefinition) -> Result<ProtoEnum> {
    let type_name = il2cpp.metadata.get_string_by_index(ty_def.nameIndex);
    let mut new_enum = ProtoEnum::create(&type_name, ty_def.byvalTypeIndex);

    // Skip the first field ("__value") and iterate over the remaining fields.
    for j in ty_def.get_field_range().skip(1) {
        let field = &il2cpp.metadata.fields[j];
        let element_name = il2cpp.metadata.get_string_by_index(field.nameIndex);
        let element_value = get_field_default_numeric_value(il2cpp, j as i32)?;
        new_enum.add_variant(&element_name, element_value);
    }

    Ok(new_enum)
}
