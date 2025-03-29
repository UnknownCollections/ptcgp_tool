//! # Unity IL2CPP Bindings via Bindgen
//!
//! This module provides Foreign Function Interface (FFI) bindings to the Unity IL2CPP runtime,
//! automatically generated by [bindgen]. It exposes Rust interfaces that wrap the Unity IL2CPP headers,
//! enabling direct interaction with the runtime.
//!
//! ## Supported Versions
//!
//! This module conditionally compiles bindings for different versions of the IL2CPP headers based on feature flags:
//!
//! - **Version 2022322f1** (enabled with the `2022322f1` feature)
//!   - Corresponds to a supported global metadata version of **29**.
//!
//! - **Version 2022356f1** (enabled with the `2022356f1` feature)
//!   - Corresponds to a supported global metadata version of **31**.
//!
//! Choose the appropriate feature while building to match the Unity IL2CPP version you want to support.
//!
//! ## Notes
//!
//! - The bindings are auto-generated and tailored to the specific version of the Unity IL2CPP headers.
//! - If your project uses a different Unity IL2CPP version or requires additional customization,
//!   you may need to adjust the generated bindings accordingly.
//!
//! [bindgen]: https://github.com/rust-lang/rust-bindgen


mod il2cpp_2022322f1;
mod il2cpp_2022356f1;

#[cfg(feature = "2022322f1")]
pub use il2cpp_2022322f1::*;
#[cfg(feature = "2022322f1")]
pub const SUPPORTED_GLOBAL_METADATA_VERSION: i32 = 29;
#[cfg(feature = "2022322f1")]
pub const SUPPORTED_VERSION_NAME: &str = "2022.3.22f1";

#[cfg(feature = "2022356f1")]
pub use il2cpp_2022356f1::*;
#[cfg(feature = "2022356f1")]
pub const SUPPORTED_GLOBAL_METADATA_VERSION: i32 = 31;
#[cfg(feature = "2022356f1")]
pub const SUPPORTED_VERSION_NAME: &str = "2022.3.56f1";