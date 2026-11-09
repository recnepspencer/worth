//! Checked storage for an independent retained output and its delta construction.
use super::{AspectPatchStorageBounds, AuthoritativeRecordAspectPatch};
use crate::aspects::{
    AuthoritativeRecordAspectState as State, CanonicalAspectStateMap as Map,
    ContractValidatedAspectValue as Value, ContractValidatedAspectValueView as View,
};

impl AuthoritativeRecordAspectPatch {
    /// Reachable backing retained by a successor storage holder, plus copy-path
    /// scratch. This is not a physical Clone invoice: immutable payloads share.
    /// The caller may retire a predecessor only after admitting the successor.
    /// Quotes inspect changed keys and cached extents, never all record entries.
    pub fn application_retained_storage_bounds(
        &self,
        current: Option<&State>,
    ) -> Option<AspectPatchStorageBounds> {
        let map = current.map(State::aspects);
        let mut payload = map.map_or(0, Map::payload_storage_bytes);
        let mut count = map.map_or(0, Map::len);
        let mut copied = 0usize;
        for contract in &self.whole_aspect_clears {
            if let Some(value) = current.and_then(|state| state.get(contract.key())) {
                payload = payload.checked_sub(entry_bytes(value)?)?;
                count = count.checked_sub(1)?;
            }
        }
        let mut new_key_width = 0usize;
        for value in &self.whole_aspect_sets {
            let was_cleared = self
                .whole_aspect_clears
                .binary_search_by(|contract| contract.key().cmp(value.key()))
                .is_ok();
            if !was_cleared && let Some(previous) = current.and_then(|state| state.get(value.key()))
            {
                payload = payload.checked_sub(entry_bytes(previous)?)?;
                count = count.checked_sub(1)?;
            }
            count = count.checked_add(1)?;
            payload = payload.checked_add(entry_bytes(value)?)?;
            copied = copied.checked_add(value.key().as_str().len())?;
            new_key_width = new_key_width.max(value.key().as_str().len());
        }
        let mut denial_key = 0usize;
        for patch in &self.field_patches {
            denial_key = denial_key.max(patch.key().as_str().len());
            for key in patch
                .field_sets
                .iter()
                .map(|(key, _)| key)
                .chain(patch.field_clears.iter())
            {
                denial_key = denial_key.max(key.as_str().len());
            }
            let Some(previous) = current.and_then(|state| state.get(patch.key())) else {
                continue; // Application returns its existing typed missing-aspect denial.
            };
            let View::Struct(fields) = previous.view() else {
                continue; // Application returns its existing typed struct-required denial.
            };
            // A rebind validates its whole new definition. An unchanged admitted
            // definition only validates changed fields and needs no shape scan.
            if !previous.contract().shares_definition(patch.contract())
                && let crate::aspects::AspectShape::Struct(shape) = patch.contract().shape()
            {
                for declaration in shape.fields() {
                    denial_key = denial_key.max(declaration.key().as_str().len());
                }
            }
            let (retained_fields, field_scratch) =
                fields.patched_storage_bounds(&patch.field_sets, &patch.field_clears)?;
            let retained_value =
                Value::struct_retained_storage_bytes(patch.contract(), retained_fields)?;
            payload = payload
                .checked_sub(entry_bytes(previous)?)?
                .checked_add(std::mem::size_of::<(crate::aspects::AspectKey, Value)>() as u128)?
                .checked_add(patch.key().as_str().len() as u128)?
                .checked_add(retained_value as u128)?;
            copied = copied
                .checked_add(field_scratch)?
                .checked_add(Value::admission_storage_bytes(patch.contract())?)?
                .checked_add(patch.key().as_str().len())?;
        }
        if current.is_none() && self.whole_aspect_sets.is_empty() {
            return Some(AspectPatchStorageBounds {
                retained_bytes: 0,
                transient_bytes: denial_key,
            });
        }
        let operations = self
            .whole_aspect_clears
            .len()
            .checked_add(self.whole_aspect_sets.len().checked_mul(2)?)?
            .checked_add(self.field_patches.len().checked_mul(2)?)?;
        let path_bytes = match map {
            Some(map) => {
                map.edit_storage_bytes(operations, self.whole_aspect_sets.len(), new_key_width)?
            }
            None => Map::empty_edit_storage_bytes(
                operations,
                self.whole_aspect_sets.len(),
                new_key_width,
            )?,
        };
        // Copy-path scratch includes final nodes as well as temporary nodes.
        // This intentional overlap is a conservative bound, not an exact invoice.
        Some(AspectPatchStorageBounds {
            retained_bytes: usize::try_from(payload)
                .ok()?
                .checked_add(Map::construction_storage_bytes(count)?)?,
            transient_bytes: path_bytes.checked_add(copied)?.checked_add(denial_key)?,
        })
    }
}

fn entry_bytes(value: &Value) -> Option<u128> {
    Some(
        std::mem::size_of::<(crate::aspects::AspectKey, Value)>() as u128
            + value.key().owned_allocation_capacity_bytes() as u128
            + value.owned_allocation_capacity_bytes() as u128,
    )
}
