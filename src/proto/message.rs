use crate::proto::field::ProtoField;
use crate::proto::map::ProtoMapField;
use crate::proto::one_of::ProtoOneOf;
use crate::proto::proto_enum::ProtoEnum;
use crate::unity::generated::CIl2Cpp::TypeIndex;

/// Represents a protocol buffer message.
///
/// A message is a composite data structure that can contain fields, oneof groups, map fields,
/// nested messages, and nested enums. It is uniquely identified by its type index.
#[derive(Clone, PartialEq)]
pub struct ProtoMessage {
    /// The name of the message.
    pub name: String,
    /// The list of fields in the message.
    pub fields: Vec<ProtoField>,
    /// The list of oneof groups in the message.
    pub oneofs: Vec<ProtoOneOf>,
    /// The list of map fields in the message.
    pub map_fields: Vec<ProtoMapField>,
    /// Nested messages contained within this message.
    pub nested_messages: Vec<ProtoMessage>,
    /// Nested enums contained within this message.
    pub nested_enums: Vec<ProtoEnum>,
    /// The unique type index assigned to the message.
    pub type_index: TypeIndex,
}

impl ProtoMessage {
    /// Creates a new `ProtoMessage` with the specified name and type index.
    ///
    /// Initializes all component lists (fields, oneofs, map_fields, nested messages, and nested enums) as empty.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the message.
    /// * `type_index` - The unique type index for the message.
    ///
    /// # Returns
    ///
    /// A new instance of `ProtoMessage`.
    pub fn create<S: Into<String>>(name: S, type_index: TypeIndex) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            oneofs: Vec::new(),
            map_fields: Vec::new(),
            nested_messages: Vec::new(),
            nested_enums: Vec::new(),
            type_index,
        }
    }

    /// Merges another `ProtoMessage` into this one.
    ///
    /// This operation appends fields, oneof groups, map fields, and nested enums.
    /// Nested messages with the same type index are merged recursively.
    ///
    /// # Arguments
    ///
    /// * `other` - The `ProtoMessage` to merge.
    pub fn merge(&mut self, other: ProtoMessage) {
        self.fields.extend(other.fields);
        self.oneofs.extend(other.oneofs);
        self.map_fields.extend(other.map_fields);
        self.nested_enums.extend(other.nested_enums);

        for nested in other.nested_messages {
            if let Some(existing) = self
                .nested_messages
                .iter_mut()
                .find(|m| m.type_index == nested.type_index)
            {
                existing.merge(nested);
            } else {
                self.nested_messages.push(nested);
            }
        }
    }

    /// Adds a field to the message.
    ///
    /// # Arguments
    ///
    /// * `field` - The `ProtoField` to be added.
    pub fn add_field(&mut self, field: ProtoField) {
        self.fields.push(field);
    }

    /// Adds a oneof group to the message.
    ///
    /// # Arguments
    ///
    /// * `oneof` - The `ProtoOneOf` group to be added.
    pub fn add_oneof(&mut self, oneof: ProtoOneOf) {
        self.oneofs.push(oneof);
    }

    /// Adds a map field to the message.
    ///
    /// # Arguments
    ///
    /// * `map` - The `ProtoMapField` to be added.
    pub fn add_map_field(&mut self, map: ProtoMapField) {
        self.map_fields.push(map);
    }

    /// Retrieves the type indices defined within this message.
    ///
    /// This includes the message’s own type index along with those from nested enums and messages.
    ///
    /// # Returns
    ///
    /// A vector of `TypeIndex` values representing the contained types.
    pub fn get_contained_types(&self) -> Vec<TypeIndex> {
        let mut contained_types = Vec::new();
        contained_types.push(self.type_index);
        for en in &self.nested_enums {
            contained_types.push(en.type_index);
        }
        for msg in &self.nested_messages {
            contained_types.extend(msg.get_contained_types());
        }
        contained_types
    }

    /// Retrieves the type indices that this message uses.
    ///
    /// This is determined by scanning fields, oneof groups, map fields, nested messages, and nested enums.
    ///
    /// # Returns
    ///
    /// A vector of `TypeIndex` values representing the used types.
    pub fn get_used_types(&self) -> Vec<TypeIndex> {
        let mut used_types = Vec::new();
        for field in &self.fields {
            if let Some(field_type_index) = field.field_type_index {
                used_types.push(field_type_index);
            }
        }
        for oneof in &self.oneofs {
            for field in &oneof.fields {
                if let Some(field_type_index) = field.field_type_index {
                    used_types.push(field_type_index);
                }
            }
        }
        for map in &self.map_fields {
            if let Some(key_type_index) = map.key_type_index {
                used_types.push(key_type_index);
            }
            if let Some(value_type_index) = map.value_type_index {
                used_types.push(value_type_index);
            }
        }
        for nested in &self.nested_messages {
            used_types.extend(nested.get_used_types());
        }
        for nested in &self.nested_enums {
            used_types.push(nested.type_index);
        }

        used_types
    }
}
