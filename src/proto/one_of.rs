use crate::proto::field::{ProtoCardinality, ProtoField};

/// Represents a `oneof` group within a protocol buffer message.
///
/// A oneof group allows multiple fields to be defined such that only one field
/// can be set at any given time.
#[derive(Clone, PartialEq)]
pub struct ProtoOneOf {
    /// The name of the oneof group.
    pub name: String,
    /// The fields belonging to this oneof group.
    pub fields: Vec<ProtoField>,
}

impl ProtoOneOf {
    /// Creates a new `ProtoOneOf` with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to assign to the oneof group.
    ///
    /// # Returns
    ///
    /// A new instance of `ProtoOneOf` with no fields.
    pub fn create(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
        }
    }

    /// Adds a field to the oneof group.
    ///
    /// Forces the field's cardinality to `Single` since oneof fields cannot be repeated or optional.
    ///
    /// # Arguments
    ///
    /// * `field` - The `ProtoField` to add.
    pub fn add_field(&mut self, mut field: ProtoField) {
        field.cardinality = ProtoCardinality::Single;
        self.fields.push(field);
    }
}
