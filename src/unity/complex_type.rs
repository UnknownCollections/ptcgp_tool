use crate::unity::generated::CIl2Cpp::TypeIndex;
use std::fmt::{Display, Formatter};

/// Represents a collection of complex type arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexTypeArgs {
    /// A vector holding each complex type argument.
    pub args: Vec<ComplexType>,
}

impl ComplexTypeArgs {
    /// Creates a new `ComplexTypeArgs` from a vector of `ComplexType`.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector containing complex types.
    pub fn new(args: Vec<ComplexType>) -> Self {
        ComplexTypeArgs { args }
    }

    /// Returns a comma-separated string representation of all complex type arguments.
    ///
    /// If `with_namespace` is `true`, each type's namespace is included.
    ///
    /// # Errors
    ///
    /// Returns a formatting error if writing to the string fails.
    pub fn get_name_str(&self, with_namespace: bool) -> Result<String, std::fmt::Error> {
        let res = self
            .args
            .iter()
            .map(|arg| arg.get_name_str(with_namespace))
            .collect::<Result<Vec<String>, std::fmt::Error>>()?
            .join(", ");
        Ok(res)
    }

    /// Retrieves the first available module name from the type arguments.
    ///
    /// It iterates over the arguments and returns the module name from the first
    /// `Simple` type that has a module specified.
    pub fn get_module_name(&self) -> Option<String> {
        self.args.iter().find_map(|arg| match arg {
            ComplexType::Simple {
                module: Some(module),
                ..
            } => Some(module.clone()),
            _ => None,
        })
    }
}

impl Display for ComplexTypeArgs {
    /// Formats the complex type arguments as a string including their namespaces.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name_str(true)?)
    }
}

/// Represents a namespace for a complex type, which can be either a simple string
/// or a nested complex type.
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexTypeNamespace {
    /// A simple namespace represented as a string.
    Simple(String),
    /// A complex namespace represented by a nested `ComplexType`.
    Complex(Box<ComplexType>),
}

impl Display for ComplexTypeNamespace {
    /// Formats the namespace as a string, delegating to the inner representation.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexTypeNamespace::Simple(s) => write!(f, "{}", s),
            ComplexTypeNamespace::Complex(inner) => write!(f, "{}", inner),
        }
    }
}

/// Represents various forms of complex types, including simple types, pointers,
/// arrays, and generics.
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexType {
    /// A basic type with optional module, namespace, type index, and name.
    Simple {
        /// Optional module name.
        module: Option<String>,
        /// Optional namespace associated with the type.
        namespace: Option<ComplexTypeNamespace>,
        /// Optional type index from Unity's type system.
        type_index: Option<TypeIndex>,
        /// The name of the type.
        name: String,
    },
    /// A pointer type (e.g. `Foo*`).
    Pointer(Box<ComplexType>),
    /// An array type (e.g. `Foo[]`).
    Array(Box<ComplexType>),
    /// A generic type with a base type and a set of type arguments (e.g. `Type<Arg1, Arg2>`).
    Generic {
        /// The base type for the generic type.
        base: Box<ComplexType>,
        /// The arguments provided to the generic type.
        args: ComplexTypeArgs,
    },
}

impl ComplexType {
    /// Retrieves the namespace of the complex type if available.
    ///
    /// For composite types like pointers, arrays, or generics, the method delegates
    /// to the underlying type.
    pub fn get_namespace(&self) -> Option<&str> {
        match self {
            ComplexType::Simple {
                namespace: Some(ns),
                ..
            } => match ns {
                ComplexTypeNamespace::Simple(s) => Some(s),
                ComplexTypeNamespace::Complex(inner) => inner.get_namespace(),
            },
            ComplexType::Pointer(inner) | ComplexType::Array(inner) => inner.get_namespace(),
            ComplexType::Generic { base, .. } => base.get_namespace(),
            _ => None,
        }
    }

    /// Retrieves the root namespace of the complex type, if present.
    ///
    /// This function applies only to `Simple` types with an associated namespace.
    pub fn get_root_namespace(&self) -> Option<&str> {
        match self {
            ComplexType::Simple {
                namespace: Some(ns),
                ..
            } => match ns {
                ComplexTypeNamespace::Simple(s) => Some(s),
                ComplexTypeNamespace::Complex(c) => c.get_root_namespace_internal(),
            },
            _ => None,
        }
    }

    /// Internal helper to recursively determine the root namespace for nested types.
    ///
    /// If no explicit namespace is found, it defaults to using the type's name.
    fn get_root_namespace_internal(&self) -> Option<&str> {
        match self {
            ComplexType::Simple {
                namespace: Some(ns),
                ..
            } => match ns {
                ComplexTypeNamespace::Simple(s) => Some(s),
                ComplexTypeNamespace::Complex(inner) => inner.get_root_namespace_internal(),
            },
            // If no namespace is provided, use the type's name as the fallback.
            ComplexType::Simple { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Returns a formatted string representation of the complex type.
    ///
    /// If `with_namespace` is `true`, the namespace (if any) is included in the output.
    ///
    /// # Errors
    ///
    /// Returns a formatting error if writing to the string fails.
    pub fn get_name_str(&self, with_namespace: bool) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;

        let mut s = String::with_capacity(64);
        match self {
            ComplexType::Simple {
                module: _,
                namespace,
                name,
                ..
            } => {
                if let Some(ns) = namespace {
                    if with_namespace {
                        write!(s, "{}.{}", ns, name)?
                    } else {
                        write!(s, "{}", name)?
                    }
                } else {
                    write!(s, "{}", name)?
                }
            }
            ComplexType::Pointer(inner) => write!(s, "{}*", inner)?,
            ComplexType::Array(inner) => write!(s, "{}[]", inner)?,
            ComplexType::Generic { base, args } => {
                write!(s, "{}<{args}>", base.get_name_str(with_namespace)?)?
            }
        }
        Ok(s)
    }

    /// Retrieves the type index from a `Simple` complex type.
    ///
    /// For pointer and array types, it delegates to the underlying type.
    /// Returns `None` for generic types as nested type indexes are not yet supported.
    pub fn get_type_index(&self) -> Option<TypeIndex> {
        match self {
            ComplexType::Simple {
                type_index: Some(type_index),
                ..
            } => Some(*type_index),
            ComplexType::Pointer(inner) => inner.get_type_index(),
            ComplexType::Array(inner) => inner.get_type_index(),
            ComplexType::Generic { .. } => {
                unimplemented!("Nested type indexes aren't yet supported")
            }
            _ => None,
        }
    }
}

impl Display for ComplexType {
    /// Formats the complex type as a string including its namespace.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name_str(true)?)
    }
}
