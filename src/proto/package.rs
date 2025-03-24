use crate::proto::circular::{messages_to_message_groups, ProtoMessageGroups};
use crate::proto::message::ProtoMessage;
use crate::proto::proto_enum::ProtoEnum;
use crate::proto::service::ProtoService;
use crate::unity::generated::CIl2Cpp::TypeIndex;
use nohash_hasher::IntSet;

/// Represents a protocol buffer package.
///
/// A package encapsulates enums, messages, and services that share a common namespace.
/// It also handles grouping of messages (especially for circular dependencies) and tracks
/// type usage within the package.
#[derive(Clone, Default, PartialEq)]
pub struct ProtoPackage {
    /// Indicates whether the package has been sealed, preventing further modifications.
    is_sealed: bool,

    /// The name of the package.
    pub package_name: String,
    /// Header comments associated with the package.
    pub header_comments: Vec<String>,
    /// Enumerations defined in the package.
    pub enums: Vec<ProtoEnum>,
    /// Messages added to the package before sealing.
    messages: Vec<ProtoMessage>,
    /// Groups of messages, typically organized based on circular dependencies.
    pub msg_groups: Option<ProtoMessageGroups>,
    /// Services defined in the package.
    pub services: Vec<ProtoService>,
    /// Set of type indices used by the package.
    pub used_types: IntSet<TypeIndex>,
    /// Set of type indices contained within the package.
    pub contained_types: IntSet<TypeIndex>,
}

impl ProtoPackage {
    /// Creates a new `ProtoPackage` with the given name and header comments.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The name of the package.
    /// * `header_comments` - A list of header comments.
    ///
    /// # Returns
    ///
    /// A new instance of `ProtoPackage`.
    pub fn new<S: Into<String>>(package_name: S, header_comments: Vec<String>) -> Self {
        Self {
            is_sealed: false,
            package_name: package_name.into(),
            header_comments,
            enums: Vec::new(),
            messages: Vec::new(),
            msg_groups: None,
            services: Vec::new(),
            used_types: IntSet::default(),
            contained_types: IntSet::default(),
        }
    }

    /// Adds an enumeration to the package.
    ///
    /// # Arguments
    ///
    /// * `en` - The `ProtoEnum` to add.
    ///
    /// # Panics
    ///
    /// Panics if the package has been sealed.
    pub fn add_enum(&mut self, en: ProtoEnum) {
        if self.is_sealed {
            panic!("Cannot add enum to sealed package");
        }
        self.enums.push(en);
    }

    /// Adds a message to the package.
    ///
    /// # Arguments
    ///
    /// * `msg` - The `ProtoMessage` to add.
    ///
    /// # Panics
    ///
    /// Panics if the package has been sealed.
    pub fn add_message(&mut self, msg: ProtoMessage) {
        if self.is_sealed {
            panic!("Cannot add message to sealed package");
        }
        self.messages.push(msg);
    }

    /// Adds a service to the package.
    ///
    /// # Arguments
    ///
    /// * `service` - The `ProtoService` to add.
    ///
    /// # Panics
    ///
    /// Panics if the package has been sealed.
    pub fn add_service(&mut self, service: ProtoService) {
        if self.is_sealed {
            panic!("Cannot add service to sealed package");
        }
        self.services.push(service);
    }

    /// Checks whether the package is empty (contains no enums, messages, or services).
    ///
    /// # Returns
    ///
    /// `true` if the package is empty; otherwise, `false`.
    pub fn is_empty(&self) -> bool {
        self.enums.is_empty()
            && self.messages.is_empty()
            && (self.msg_groups.as_ref().is_none_or(|g| g.is_empty()))
            && self.services.is_empty()
    }

    /// Seals the package, finalizing its contents and processing message groups.
    ///
    /// Once sealed, no further modifications (such as adding messages or enums) are allowed.
    pub fn seal(&mut self) {
        self.is_sealed = true;
        let messages = std::mem::take(&mut self.messages);
        self.msg_groups = Some(messages_to_message_groups(messages));
        self.store_types();
    }

    /// Aggregates type indices from enums, messages, and services, storing them for later use.
    fn store_types(&mut self) {
        for en in &self.enums {
            self.contained_types.insert(en.type_index);
        }
        for msg in self.msg_groups.as_ref().unwrap().iter() {
            self.contained_types.extend(msg.get_contained_types());
            self.used_types.extend(msg.get_used_types());
        }
        for svc in &self.services {
            self.contained_types.insert(svc.type_index);
            for method in &svc.methods {
                if let Some(type_index) = method.input_type_index {
                    self.used_types.insert(type_index);
                }
                if let Some(type_index) = method.output_type_index {
                    self.used_types.insert(type_index);
                }
            }
        }
    }

    /// Returns a reference to the messages added to the package.
    ///
    /// # Panics
    ///
    /// Panics if the package has been sealed.
    pub fn messages(&self) -> &[ProtoMessage] {
        if self.is_sealed {
            panic!("Cannot access messages of sealed package");
        }
        &self.messages
    }

    /// Returns a mutable reference to the messages added to the package.
    ///
    /// # Panics
    ///
    /// Panics if the package has been sealed.
    pub fn messages_mut(&mut self) -> &mut [ProtoMessage] {
        if self.is_sealed {
            panic!("Cannot access messages of sealed package");
        }
        &mut self.messages
    }
}
