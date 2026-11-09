//! Delta-only retained extent and copy-path scratch for immutable field edits.
use super::{field_extent, AspectValue, FieldKey, StructAspectValue};

impl StructAspectValue {
    pub(in crate::aspects) fn patched_storage_bounds(
        &self,
        sets: &[(FieldKey, AspectValue)],
        clears: &[FieldKey],
    ) -> Option<(usize, usize)> {
        let mut retained = self.allocation_bytes;
        for key in clears {
            if let Some((stored_key, value)) = self.fields.get_key_value(key) {
                retained = retained.checked_sub(field_extent(stored_key, value).0)?;
            }
        }
        let mut copied_payload = 0usize;
        for (key, value) in sets {
            if clears.binary_search(key).is_err()
                && let Some((stored_key, previous)) = self.fields.get_key_value(key)
            {
                retained = retained.checked_sub(field_extent(stored_key, previous).0)?;
            }
            retained = retained.checked_add(field_extent(key, value).0)?;
            copied_payload = copied_payload
                .checked_add(key.as_str().len())?
                .checked_add(value.clone_allocation_bytes()?)?;
        }
        let operations = clears.len().checked_add(sets.len().checked_mul(2)?)?;
        let scratch = self
            .fields
            .edit_storage_bytes(operations, sets.len())?
            .checked_add(copied_payload)?;
        Some((usize::try_from(retained).ok()?, scratch))
    }
}
