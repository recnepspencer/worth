use std::collections::BTreeMap;

use super::{FieldDeclaration, FieldKey};
use crate::values::AspectValue;
use serde::{Deserialize, Serialize};

mod field_map;
mod storage_bounds;
pub use storage_bounds::StructAspectValueStorageBounds;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructAspectShape {
    fields: Vec<FieldDeclaration>,
}

impl StructAspectShape {
    pub fn new(fields: impl IntoIterator<Item = FieldDeclaration>) -> Option<Self> {
        let mut fields: Vec<_> = fields.into_iter().collect();
        fields.sort_by(|left, right| left.key().cmp(right.key()));

        if fields.windows(2).any(|pair| pair[0].key() == pair[1].key()) {
            return None;
        }

        Some(Self { fields })
    }

    pub(crate) fn clone_allocation_bytes(&self) -> Option<usize> {
        let mut bytes = std::alloc::Layout::array::<FieldDeclaration>(self.fields.len())
            .ok()?
            .size();
        for field in &self.fields {
            bytes = bytes.checked_add(field.key().as_str().len())?;
        }
        Some(bytes)
    }

    pub fn fields(&self) -> &[FieldDeclaration] {
        &self.fields
    }

    pub fn field(&self, key: &FieldKey) -> Option<&FieldDeclaration> {
        self.fields.iter().find(|field| field.key() == key)
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.fields
            .capacity()
            .saturating_mul(std::mem::size_of::<FieldDeclaration>())
            .saturating_add(
                self.fields
                    .iter()
                    .map(|field| field.key().owned_allocation_capacity_bytes())
                    .sum(),
            )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructAspectValue {
    #[serde(
        serialize_with = "field_map::serialize",
        deserialize_with = "field_map::deserialize"
    )]
    fields: Box<[(FieldKey, AspectValue)]>,
}

impl StructAspectValue {
    pub(crate) fn clone_allocation_bytes(&self) -> Option<usize> {
        let mut bytes = std::alloc::Layout::array::<(FieldKey, AspectValue)>(self.fields.len())
            .ok()?
            .size();
        for (key, value) in &self.fields {
            bytes = bytes
                .checked_add(key.as_str().len())?
                .checked_add(value.clone_allocation_bytes()?)?;
        }
        Some(bytes)
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn new(
        fields: impl IntoIterator<Item = (FieldKey, AspectValue)>,
    ) -> Result<Self, StructAspectValueConstructionDenial> {
        let mut canonical_fields = BTreeMap::new();
        for (field_key, field_value) in fields {
            if canonical_fields
                .insert(field_key.clone(), field_value)
                .is_some()
            {
                return Err(StructAspectValueConstructionDenial::DuplicateField(
                    field_key,
                ));
            }
        }

        Ok(Self {
            fields: canonical_fields.into_iter().collect(),
        })
    }

    /// Consumes strictly key-ordered, unique fields in O(n), without allocating.
    /// This constructs value meaning; contract admission remains separate.
    pub fn from_sorted_fields(
        fields: Box<[(FieldKey, AspectValue)]>,
    ) -> Result<Self, StructAspectValueSortedFieldsDenied> {
        if let Some(invalid_pair_index) = fields.windows(2).position(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(StructAspectValueSortedFieldsDenied {
                fields,
                invalid_pair_index,
            });
        }
        Ok(Self { fields })
    }

    /// Checked backing-storage bounds for decoding this many map entries.
    /// Key/value-owned heaps are excluded and must be quoted separately.
    pub fn decode_storage_bounds(entry_count: usize) -> Option<StructAspectValueStorageBounds> {
        storage_bounds::decode(entry_count)
    }

    pub fn fields(&self) -> impl Iterator<Item = (&FieldKey, &AspectValue)> {
        self.fields.iter().map(|(key, value)| (key, value))
    }

    pub fn get(&self, key: &FieldKey) -> Option<&AspectValue> {
        self.fields
            .binary_search_by(|(field, _)| field.cmp(key))
            .ok()
            .map(|index| &self.fields[index].1)
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.fields.iter().fold(
            self.fields
                .len()
                .saturating_mul(std::mem::size_of::<(FieldKey, AspectValue)>()),
            |bytes, (key, value)| {
                bytes
                    .saturating_add(key.owned_allocation_capacity_bytes())
                    .saturating_add(value.owned_allocation_capacity_bytes())
            },
        )
    }
}

/// Original owned fields rejected for nonascending or duplicate adjacent keys.
#[derive(Debug)]
pub struct StructAspectValueSortedFieldsDenied {
    fields: Box<[(FieldKey, AspectValue)]>,
    invalid_pair_index: usize,
}

impl StructAspectValueSortedFieldsDenied {
    pub fn fields(&self) -> &[(FieldKey, AspectValue)] {
        &self.fields
    }

    /// Index of the left field in the first invalid adjacent pair.
    pub fn invalid_pair_index(&self) -> usize {
        self.invalid_pair_index
    }

    pub fn into_fields(self) -> Box<[(FieldKey, AspectValue)]> {
        self.fields
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructAspectValueConstructionDenial {
    DuplicateField(FieldKey),
}
