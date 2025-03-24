use crate::unity::generated::CIl2Cpp::TypeIndex;

/// Represents a gRPC service in a protocol buffer definition.
///
/// A service defines a set of RPC methods that can be invoked remotely.
#[derive(Clone, PartialEq)]
pub struct ProtoService {
    /// The name of the service.
    pub name: String,
    /// The unique type index associated with this service.
    pub type_index: TypeIndex,
    /// The list of RPC methods provided by the service.
    pub methods: Vec<ProtoServiceMethod>,
}

impl ProtoService {
    /// Creates a new `ProtoService` with the specified name and type index.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the service.
    /// * `type_index` - The unique type index for the service.
    ///
    /// # Returns
    ///
    /// A new instance of `ProtoService`.
    pub fn new<S: Into<String>>(name: S, type_index: TypeIndex) -> Self {
        Self {
            name: name.into(),
            type_index,
            methods: Vec::new(),
        }
    }

    /// Adds an RPC method to the service.
    ///
    /// # Arguments
    ///
    /// * `method` - The `ProtoServiceMethod` to add.
    pub fn add_method(&mut self, method: ProtoServiceMethod) {
        self.methods.push(method);
    }

    /// Retrieves a list of type indices used by the service's RPC methods.
    ///
    /// Includes both input and output types.
    ///
    /// # Returns
    ///
    /// A vector of `TypeIndex` values representing the used types.
    pub fn get_used_types(&self) -> Vec<TypeIndex> {
        let mut used_types = Vec::new();
        for method in &self.methods {
            if let Some(input_type_index) = method.input_type_index {
                used_types.push(input_type_index);
            }
            if let Some(input_type_index) = method.output_type_index {
                used_types.push(input_type_index);
            }
        }

        used_types
    }
}

/// Represents an RPC method in a gRPC service.
///
/// Each method specifies its name, input/output types, and streaming options.
#[derive(Clone, PartialEq)]
pub struct ProtoServiceMethod {
    /// The name of the RPC method.
    pub name: String,
    /// The namespace where the input type is defined.
    pub input_namespace: Option<String>,
    /// The input type of the RPC.
    pub input_type: String,
    /// An optional index representing the input type.
    pub input_type_index: Option<TypeIndex>,
    /// The namespace where the output type is defined.
    pub output_namespace: Option<String>,
    /// The output type of the RPC.
    pub output_type: String,
    /// An optional index representing the output type.
    pub output_type_index: Option<TypeIndex>,
    /// Indicates whether the RPC accepts a stream of messages.
    pub client_streaming: bool,
    /// Indicates whether the RPC returns a stream of messages.
    pub server_streaming: bool,
}

impl ProtoServiceMethod {
    /// Creates a new `ProtoServiceMethod`.
    ///
    /// Initializes all properties including namespaces, types, type indices, and streaming flags.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the RPC method.
    /// * `input_namespace` - Optional namespace for the input type.
    /// * `input_type` - The input type as a string.
    /// * `input_type_index` - Optional index for the input type.
    /// * `output_namespace` - Optional namespace for the output type.
    /// * `output_type` - The output type as a string.
    /// * `output_type_index` - Optional index for the output type.
    /// * `client_streaming` - If `true`, the RPC accepts a stream of input messages.
    /// * `server_streaming` - If `true`, the RPC returns a stream of output messages.
    ///
    /// # Returns
    ///
    /// A new instance of `ProtoServiceMethod`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        input_namespace: Option<String>,
        input_type: String,
        input_type_index: Option<TypeIndex>,
        output_namespace: Option<String>,
        output_type: String,
        output_type_index: Option<TypeIndex>,
        client_streaming: bool,
        server_streaming: bool,
    ) -> Self {
        Self {
            name,
            input_namespace,
            input_type,
            input_type_index,
            output_namespace,
            output_type,
            output_type_index,
            client_streaming,
            server_streaming,
        }
    }
}
