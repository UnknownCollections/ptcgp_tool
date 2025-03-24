use crate::proto::message::ProtoMessage;
use crate::proto::package::ProtoPackage;
use crate::proto::writer::{format_package_filename, ProtoGenFile};
use crate::unity::generated::CIl2Cpp::TypeIndex;
use anyhow::{anyhow, Result};
use hashbrown::{HashMap, HashSet};
use nohash_hasher::{IntMap, IntSet};

/// Represents the set of generated files for enums, messages, and services.
#[derive(Default)]
pub struct ProtoGenSchema {
    /// Generated files for enumerations.
    pub enums: Vec<ProtoGenFile>,
    /// Generated files for messages.
    pub messages: Vec<ProtoGenFile>,
    /// Generated files for services.
    pub services: Vec<ProtoGenFile>,
}

/// Represents the schema for protocol buffer definitions.
///
/// Manages packages and builds a mapping from type indices to output filenames.
#[derive(Default)]
pub struct ProtoSchema {
    /// Mapping from package names to their corresponding `ProtoPackage` instances.
    pub packages: HashMap<String, ProtoPackage>,
    /// Mapping from type indices to output filenames.
    type_file_mapping: IntMap<TypeIndex, String>,
}

impl ProtoSchema {
    /// Creates a new, empty `ProtoSchema`.
    ///
    /// # Returns
    ///
    /// A new instance of `ProtoSchema`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a package into the schema.
    ///
    /// # Arguments
    ///
    /// * `package` - The `ProtoPackage` to insert.
    pub fn insert(&mut self, package: ProtoPackage) {
        self.packages.insert(package.package_name.clone(), package);
    }

    /// Retrieves a mutable reference to a package by its name.
    ///
    /// If the package does not exist, a new package is created.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The name of the package.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ProtoPackage`.
    pub fn get(&mut self, package_name: String) -> &mut ProtoPackage {
        self.packages
            .entry(package_name.clone())
            .or_insert_with(|| ProtoPackage::new(package_name, vec![]))
    }

    /// Recursively builds file mappings for a message and its nested types.
    ///
    /// Associates each type (message or nested enum) with the specified filename.
    ///
    /// # Arguments
    ///
    /// * `type_file_mapping` - The mapping from type indices to filenames.
    /// * `filename` - The filename to associate with the message and its nested types.
    /// * `msg` - The `ProtoMessage` to process.
    fn build_message_file_mappings(
        type_file_mapping: &mut IntMap<TypeIndex, String>,
        filename: String,
        msg: &ProtoMessage,
    ) {
        for nested_enum in &msg.nested_enums {
            type_file_mapping.insert(nested_enum.type_index, filename.clone());
        }
        for nested_msg in &msg.nested_messages {
            Self::build_message_file_mappings(type_file_mapping, filename.clone(), nested_msg);
        }
        type_file_mapping.insert(msg.type_index, filename);
    }

    /// Builds file mappings for all types contained in a package.
    ///
    /// Processes enums, message groups, and services within the package.
    ///
    /// # Arguments
    ///
    /// * `type_file_mapping` - The mapping from type indices to filenames.
    /// * `package` - The `ProtoPackage` to process.
    fn build_package_file_mappings(
        type_file_mapping: &mut IntMap<TypeIndex, String>,
        package: &ProtoPackage,
    ) {
        for en in &package.enums {
            type_file_mapping.insert(
                en.type_index,
                format!("{}.{}", package.package_name, en.name),
            );
        }
        if let Some(msg_groups) = &package.msg_groups {
            for msg_group in msg_groups {
                let primary_msg = msg_group.get_primary();
                let filepath = format!("{}.{}", package.package_name, primary_msg.name);
                for msg in msg_group.iter() {
                    Self::build_message_file_mappings(type_file_mapping, filepath.clone(), msg);
                }
            }
        }
        for svc in &package.services {
            type_file_mapping.insert(
                svc.type_index,
                format!("{}.{}", package.package_name, svc.name),
            );
        }
    }

    /// Seals the schema by finalizing all packages and building type-file mappings.
    ///
    /// Finalizes each package and computes output filenames for each type.
    pub fn seal(&mut self) {
        for package in self.packages.values_mut() {
            package.seal();
        }
        for package in self.packages.values_mut() {
            Self::build_package_file_mappings(&mut self.type_file_mapping, package);
        }
    }

    /// Builds the final generated schema containing files for enums, messages, and services.
    ///
    /// Filters packages based on criteria and generates pretty-printed file contents.
    ///
    /// # Returns
    ///
    /// A `ProtoGenSchema` wrapped in a `Result`, or an error if generation fails.
    pub fn build(&self) -> Result<ProtoGenSchema> {
        let all_used_types = self
            .packages
            .values()
            .flat_map(|package| package.used_types.clone())
            .collect::<IntSet<TypeIndex>>();

        let mut enums = Vec::new();
        let mut messages = Vec::new();
        let mut services = Vec::new();

        let filtered: Vec<_> = self
            .packages
            .values()
            .filter(move |package| {
                !package.package_name.starts_with("Google.")
                    && !package.is_empty()
                    && !package.contained_types.is_disjoint(&all_used_types)
            })
            .collect();

        for package in filtered {
            enums.extend(self.build_enums_for_package(package)?);
            messages.extend(self.build_messages_for_package(package)?);
            services.extend(self.build_services_for_package(package)?);
        }

        Ok(ProtoGenSchema {
            enums,
            messages,
            services,
        })
    }

    /// Generates protocol buffer files for all enums in a package.
    ///
    /// # Arguments
    ///
    /// * `package` - The `ProtoPackage` to process.
    ///
    /// # Returns
    ///
    /// A vector of `ProtoGenFile` instances representing enum definitions.
    fn build_enums_for_package(
        &self,
        package: &ProtoPackage,
    ) -> Result<Vec<ProtoGenFile>> {
        let mut files = Vec::new();
        for en in &package.enums {
            let filename = self.get_formatted_filename(en.type_index)?;
            let content = en.to_pretty_string(0);

            let file = ProtoGenFile::new(
                filename,
                &package.package_name,
                &package.header_comments,
                None,
                &content,
            )?;
            files.push(file);
        }
        Ok(files)
    }

    /// Generates protocol buffer files for message groups in a package.
    ///
    /// Each message group is processed to create a file containing the primary message and its imports.
    ///
    /// # Arguments
    ///
    /// * `package` - The `ProtoPackage` to process.
    ///
    /// # Returns
    ///
    /// A vector of `ProtoGenFile` instances representing message definitions.
    fn build_messages_for_package(
        &self,
        package: &ProtoPackage,
    ) -> Result<Vec<ProtoGenFile>> {
        let mut files = Vec::new();
        for msg_group in package.msg_groups.as_ref().unwrap() {
            let filename = self.get_formatted_filename(msg_group.get_primary().type_index)?;

            // Get all imports the message needs.
            let imports = msg_group
                .get_used_types()
                .difference(&msg_group.get_contained_types())
                .map(|idx| self.get_formatted_filename(*idx))
                .filter(|import_filename| {
                    if let Ok(import_filename) = import_filename {
                        import_filename != &filename
                    } else {
                        false
                    }
                })
                .collect::<Result<HashSet<_>, _>>()?;

            let content = msg_group.to_pretty_string(0, &package.package_name);
            let file = ProtoGenFile::new(
                filename,
                &package.package_name,
                &package.header_comments,
                Some(imports),
                &content,
            )?;
            files.push(file);
        }
        Ok(files)
    }

    /// Generates protocol buffer files for services in a package.
    ///
    /// # Arguments
    ///
    /// * `package` - The `ProtoPackage` to process.
    ///
    /// # Returns
    ///
    /// A vector of `ProtoGenFile` instances representing service definitions.
    fn build_services_for_package(
        &self,
        package: &ProtoPackage,
    ) -> Result<Vec<ProtoGenFile>> {
        let mut files = Vec::new();
        for svc in &package.services {
            let filename = self.get_formatted_filename(svc.type_index)?;

            let imports = svc
                .get_used_types()
                .iter()
                .map(|idx| self.get_formatted_filename(*idx))
                .filter(|import_filename| {
                    if let Ok(import_filename) = import_filename {
                        import_filename != &filename
                    } else {
                        false
                    }
                })
                .collect::<Result<HashSet<_>, _>>()?;

            let content = svc.to_pretty_string(0);
            let file = ProtoGenFile::new(
                filename,
                &package.package_name,
                &package.header_comments,
                Some(imports),
                &content,
            )?;
            files.push(file);
        }
        Ok(files)
    }

    /// Retrieves the formatted filename associated with a given type index.
    ///
    /// The filename is derived from an internal mapping and adjusted for built-in types.
    ///
    /// # Arguments
    ///
    /// * `type_index` - The type index for which to retrieve the filename.
    ///
    /// # Returns
    ///
    /// A formatted filename as a `String`, or an error if the type index is not found.
    fn get_formatted_filename(&self, type_index: TypeIndex) -> Result<String> {
        let filename = self
            .type_file_mapping
            .get(&type_index)
            .ok_or_else(|| anyhow!("Missing type index in schema mapping for {}", type_index))?;

        let filename = Self::remap_builtin_filenames(filename);

        Ok(filename)
    }

    /// Remaps filenames for built-in types to a standardized format.
    ///
    /// If the namespace begins with the well-known types prefix, it is reformatted.
    ///
    /// # Arguments
    ///
    /// * `namespace` - The original namespace string.
    ///
    /// # Returns
    ///
    /// A formatted filename as a `String`.
    fn remap_builtin_filenames(namespace: &str) -> String {
        if let Some(remaining) = namespace.strip_prefix("Google.Protobuf.WellKnownTypes.") {
            let last_seg = remaining.rsplit('.').next().unwrap();
            format_package_filename(&format!("Google.Protobuf.{}", last_seg))
        } else {
            format_package_filename(namespace)
        }
    }
}
