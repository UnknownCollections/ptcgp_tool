use crate::unity::generated::CIl2Cpp::TypeIndex;

/// Represents a map field in a protocol buffer message.
///
/// Defines a key-value association with specific types for keys and values.
#[derive(Clone, PartialEq)]
pub struct ProtoMapField {
    /// The name of the map field.
    pub name: String,
    /// The key type as a string.
    pub key_type: String,
    /// An optional index representing the key type.
    pub key_type_index: Option<TypeIndex>,
    /// The value type as a string.
    pub value_type: String,
    /// An optional index representing the value type.
    pub value_type_index: Option<TypeIndex>,
    /// The unique tag number of the map field.
    pub tag: i32,
}

impl ProtoMapField {
    /// Creates a new `ProtoMapField` instance.
    ///
    /// # Arguments
    ///
    /// * `key_type` - The type of the keys.
    /// * `key_type_index` - Optional index identifying the key type.
    /// * `value_type` - The type of the values.
    /// * `value_type_index` - Optional index identifying the value type.
    /// * `name` - The name of the map field.
    /// * `tag` - The unique tag number of the field.
    ///
    /// # Returns
    ///
    /// A new instance of `ProtoMapField`.
    pub fn new(
        key_type: String,
        key_type_index: Option<TypeIndex>,
        value_type: String,
        value_type_index: Option<TypeIndex>,
        name: String,
        tag: i32,
    ) -> Self {
        Self {
            name,
            key_type,
            key_type_index,
            value_type,
            value_type_index,
            tag,
        }
    }
}
