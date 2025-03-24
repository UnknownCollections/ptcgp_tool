use crate::unity::generated::CIl2Cpp::{Il2CppType, Il2CppTypeEnum};
use anyhow::{anyhow, Result};

/// Enumerates the various data types that can be stored in a blob value.
/// This enum encapsulates primitive types (e.g. integers, floats, booleans, characters),
/// composite types (arrays), as well as string values and optional type information.
#[derive(Debug, Clone)]
pub enum BlobValueData {
    /// A boolean value.
    Boolean(bool),
    /// An unsigned 1-byte integer.
    U1(u8),
    /// A signed 1-byte integer.
    I1(i8),
    /// A character value.
    Char(char),
    /// An unsigned 2-byte integer.
    U2(u16),
    /// A signed 2-byte integer.
    I2(i16),
    /// An unsigned 4-byte integer.
    U4(u32),
    /// A signed 4-byte integer.
    I4(i32),
    /// An unsigned 8-byte integer.
    U8(u64),
    /// A signed 8-byte integer.
    I8(i64),
    /// A 4-byte floating point number.
    R4(f32),
    /// An 8-byte floating point number.
    R8(f64),
    /// A string value.
    String(String),
    /// An array of blob values.
    Array(Vec<BlobValue>),
    /// An optional type index providing additional Il2Cpp type information.
    TypeIndex(Option<Il2CppType>),
}

/// Represents a value within a blob, pairing type metadata with the actual data.
#[derive(Debug, Clone)]
pub struct BlobValue {
    /// The enumeration representing the underlying Il2Cpp type.
    pub il2cpp_type_enum: Il2CppTypeEnum,
    /// Optional detailed type information for enumerated types.
    pub enum_type: Option<Il2CppType>,
    /// The actual data stored in the blob.
    pub value: BlobValueData,
}

impl BlobValue {
    /// Attempts to interpret the blob value as a numeric value and convert it to an unsigned 64-bit integer.
    ///
    /// This function supports several numeric variants:
    /// - Unsigned and signed 1-, 2-, 4-, and 8-byte integers.
    ///
    /// # Returns
    ///
    /// - `Ok(u64)` if the blob value is one of the supported numeric types.
    /// - `Err(String)` if the blob value is not a supported numeric type.
    pub fn as_num(&self) -> Result<u64> {
        match &self.value {
            BlobValueData::U1(v) => Ok(*v as u64),
            BlobValueData::I1(v) => Ok(*v as u64),
            BlobValueData::U2(v) => Ok(*v as u64),
            BlobValueData::I2(v) => Ok(*v as u64),
            BlobValueData::U4(v) => Ok(*v as u64),
            BlobValueData::I4(v) => Ok(*v as u64),
            BlobValueData::I8(v) => Ok(*v as u64),
            BlobValueData::U8(v) => Ok(*v),
            vt => Err(anyhow!("BlobValue is not a number: {:?}", vt)),
        }
    }

    /// Attempts to interpret the blob value as a floating-point number and convert it to a 64-bit float.
    ///
    /// This function supports both 4-byte (R4) and 8-byte (R8) floating point variants.
    ///
    /// # Returns
    ///
    /// - `Ok(f64)` if the blob value is one of the supported floating-point types.
    /// - `Err(String)` if the blob value is not a supported floating-point type.
    pub fn as_float(&self) -> Result<f64> {
        match &self.value {
            BlobValueData::R4(v) => Ok(*v as f64),
            BlobValueData::R8(v) => Ok(*v),
            vt => Err(anyhow!("BlobValue is not a float: {:?}", vt)),
        }
    }
}
