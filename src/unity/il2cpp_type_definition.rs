use crate::unity::complex_type::ComplexType;
use crate::unity::generated::CIl2Cpp::Il2CppTypeDefinition;
use crate::unity::il2cpp::Il2Cpp;
use std::ops::Range;

impl Il2CppTypeDefinition {
    /// Returns the range of field indices associated with this type definition.
    ///
    /// The range starts at `fieldStart` and spans `field_count` entries.
    #[inline(always)]
    pub fn get_field_range(&self) -> Range<usize> {
        let start = self.fieldStart as usize;
        start..(start + self.field_count as usize)
    }

    /// Determines whether the type definition contains a field with the specified name and type.
    ///
    /// This method iterates over the fields indicated by [`get_field_range`]. For each field, it retrieves
    /// the field's name using metadata. If the name matches the provided `name`, it then checks the field's
    /// complex type against the provided `ty_name`. The comparison supports both simple and generic type formats.
    ///
    /// # Arguments
    ///
    /// * `il2cpp` - A reference to the Il2Cpp instance that holds metadata and type information.
    /// * `name` - The expected name of the field.
    /// * `ty_name` - The expected type name of the field.
    ///
    /// # Returns
    ///
    /// Returns `true` if a field matching both the name and type is found; otherwise, returns `false`.
    pub fn has_field<'a>(&'a self, il2cpp: &'a Il2Cpp<'a>, name: &str, ty_name: &str) -> bool {
        self.get_field_range().any(|field_index| {
            let field = &il2cpp.metadata.fields[field_index];

            // Retrieve the field's name from metadata.
            let field_name = il2cpp.metadata.get_string_by_index(field.nameIndex);
            if field_name != name {
                return false;
            }
            // Check the field's complex type and compare it to the expected type name.
            match il2cpp.types[field.typeIndex as usize].get_complex_type(il2cpp) {
                Ok(ComplexType::Simple { ref name, .. }) => name == ty_name,
                Ok(ComplexType::Generic { ref base, .. }) => base.to_string() == ty_name,
                _ => false,
            }
        })
    }

    /// Determines if the type is a value type.
    pub fn is_value_type(&self) -> bool {
        (self.bitfield & (1 << 0)) != 0
    }

    /// Determines if the type is an enumeration.
    pub fn is_enum_type(&self) -> bool {
        (self.bitfield & (1 << 1)) != 0
    }

    /// Checks whether the type defines a finalizer.
    pub fn has_finalize(&self) -> bool {
        (self.bitfield & (1 << 2)) != 0
    }

    /// Checks whether the type defines a class constructor (.cctor).
    pub fn has_cctor(&self) -> bool {
        (self.bitfield & (1 << 3)) != 0
    }

    /// Determines if the type is blittable (can be directly copied in memory).
    pub fn is_blittable(&self) -> bool {
        (self.bitfield & (1 << 4)) != 0
    }

    /// Checks whether the type is an imported type.
    pub fn is_import(&self) -> bool {
        (self.bitfield & (1 << 5)) != 0
    }

    /// Retrieves the packing size for the type.
    pub fn packing_size(&self) -> u8 {
        ((self.bitfield >> 6) & 0xF) as u8
    }
}
