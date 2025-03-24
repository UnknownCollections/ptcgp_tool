use crate::unity::generated::CIl2Cpp::TypeIndex;

/// Specifies the cardinality of a protocol buffer field.
///
/// Indicates whether a field is required, optional, or repeated.
#[derive(Clone, PartialEq)]
pub enum ProtoCardinality {
    /// A required field (implied when no cardinality keyword is present).
    Single,
    /// An optional field.
    Optional,
    /// A repeated field.
    Repeated,
}

/// Represents a field in a protocol buffer message.
///
/// Contains metadata such as the field's name, type, tag number, and its cardinality.
#[derive(Clone, PartialEq)]
pub struct ProtoField {
    /// The namespace where the field's type is defined.
    pub namespace: String,
    /// The name of the field.
    pub name: String,
    /// The type of the field as a string.
    pub field_type: String,
    /// An optional index representing the field's type.
    pub field_type_index: Option<TypeIndex>,
    /// The unique tag number of the field.
    pub tag: i32,
    /// The cardinality of the field (e.g., single, optional, repeated).
    pub cardinality: ProtoCardinality,
}

impl ProtoField {
    /// Creates a new `ProtoField` instance.
    ///
    /// Normalizes the namespace if it matches `"Google.Protobuf.WellKnownTypes"`, and defaults
    /// the cardinality to `Single` if not provided.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Optional namespace string where the field's type is defined.
    /// * `name` - The name of the field.
    /// * `field_type` - The type of the field as a string.
    /// * `field_type_index` - Optional index identifying the field's type.
    /// * `tag` - The unique tag number of the field.
    /// * `cardinality` - Optional cardinality; defaults to `Single` if `None`.
    ///
    /// # Returns
    ///
    /// A new instance of `ProtoField` with normalized type information.
    pub fn new(
        namespace: Option<String>,
        name: String,
        field_type: String,
        field_type_index: Option<TypeIndex>,
        tag: i32,
        cardinality: Option<ProtoCardinality>,
    ) -> Self {
        let field = Self {
            namespace: namespace.unwrap_or_default(),
            name,
            field_type,
            field_type_index,
            tag,
            cardinality: cardinality.unwrap_or(ProtoCardinality::Single),
        };
        field.remap_builtin_type()
    }

    /// Normalizes built-in type namespaces.
    ///
    /// If the namespace is `"Google.Protobuf.WellKnownTypes"`, it is remapped to
    /// `"google.protobuf"`.
    ///
    /// # Returns
    ///
    /// A `ProtoField` instance with the normalized namespace.
    fn remap_builtin_type(mut self) -> Self {
        if self.namespace == "Google.Protobuf.WellKnownTypes" {
            self.namespace = "google.protobuf".to_string();
        }
        self
    }
}
