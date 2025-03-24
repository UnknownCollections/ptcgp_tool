use crate::unity::generated::CIl2Cpp::TypeIndex;
use hashbrown::HashMap;

/// Represents an enumeration in a protocol buffer definition.
///
/// Defines a set of named values and is identified by a unique type index.
#[derive(Clone, Default, PartialEq)]
pub struct ProtoEnum {
    /// The name of the enumeration.
    pub name: String,
    /// Mapping from variant names to their corresponding `ProtoEnumVariant`.
    pub variants: HashMap<String, ProtoEnumVariant>,
    /// The unique type index for this enumeration.
    pub type_index: TypeIndex,
}

impl ProtoEnum {
    /// Creates a new `ProtoEnum` with the specified name and type index.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the enumeration.
    /// * `type_index` - The unique type index assigned to the enum.
    ///
    /// # Returns
    ///
    /// A new instance of `ProtoEnum`.
    pub fn create(name: &str, type_index: TypeIndex) -> Self {
        Self {
            name: name.to_string(),
            variants: HashMap::new(),
            type_index,
        }
    }

    /// Adds a new variant to the enumeration.
    ///
    /// The variant's name is combined with the enum's name for uniqueness.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variant.
    /// * `number` - The numeric tag associated with the variant.
    pub fn add_variant(&mut self, name: &str, number: i32) {
        let enum_name = format!("{}_{}", self.name, name);
        self.variants
            .insert(name.into(), ProtoEnumVariant::new(&enum_name, number));
    }
}

/// Represents a variant (enum value) in a protocol buffer enumeration.
#[derive(Clone, PartialEq)]
pub struct ProtoEnumVariant {
    /// The name of the enum variant.
    pub name: String,
    /// The numeric tag associated with the variant.
    pub tag: i32,
}

impl ProtoEnumVariant {
    /// Creates a new `ProtoEnumVariant` with the specified name and tag.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the enum variant.
    /// * `tag` - The numeric tag for the variant.
    ///
    /// # Returns
    ///
    /// A new instance of `ProtoEnumVariant`.
    pub fn new<S: Into<String>>(name: S, tag: i32) -> Self {
        Self {
            name: name.into(),
            tag,
        }
    }
}
