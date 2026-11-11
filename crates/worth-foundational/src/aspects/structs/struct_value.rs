use super::{FieldDeclaration, FieldKey};
use crate::values::AspectValue;
use serde::{Deserialize, Serialize};

mod field_map;
mod patch_storage;
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

    pub fn fields(&self) -> &[FieldDeclaration] {
        &self.fields
    }

    pub fn field(&self, key: &FieldKey) -> Option<&FieldDeclaration> {
        self.fields
            .binary_search_by(|field| field.key().cmp(key))
            .ok()
            .map(|index| &self.fields[index])
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

#[derive(Clone, Serialize)]
pub struct StructAspectValue {
    fields: super::field_tree::StructFields,
    #[serde(skip)]
    allocation_bytes: u128,
    #[serde(skip)]
    semantic_bytes: u128,
}
impl PartialEq for StructAspectValue {
    fn eq(&self, other: &Self) -> bool {
        self.fields == other.fields
    }
}
impl Eq for StructAspectValue {}
impl std::fmt::Debug for StructAspectValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StructAspectValue")
            .field("fields", &self.fields)
            .finish()
    }
}
impl<'de> Deserialize<'de> for StructAspectValue {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Image {
            #[serde(deserialize_with = "field_map::deserialize")]
            fields: Box<[(FieldKey, AspectValue)]>,
        }
        Self::from_sorted_fields(Image::deserialize(decoder)?.fields)
            .map_err(|error| serde::de::Error::custom(format!("{error:?}")))
    }
}
impl StructAspectValue {
    /// Cloning shares the immutable root; it allocates no heap backing.
    pub(crate) fn clone_allocation_bytes(&self) -> Option<usize> {
        Some(0)
    }

    /// Admit strictly ordered fields and move them into a balanced tree in O(n).
    /// Invalid input is returned without consuming or reallocating its backing.
    /// Success allocates tree/key/value Arc backing, but does not clone payloads.
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
        let (allocation_bytes, semantic_bytes) =
            fields
                .iter()
                .fold((0u128, 0u128), |(allocation, semantic), (key, value)| {
                    let (next_allocation, next_semantic) = field_extent(key, value);
                    (allocation + next_allocation, semantic + next_semantic)
                });
        Ok(Self {
            fields: super::field_tree::StructFields::from_sorted_fields(
                fields.into_vec().into_iter(),
            ),
            allocation_bytes,
            semantic_bytes,
        })
    }

    /// New tree/key/value Arc backing for sorted admission. Existing input and
    /// nested key/value heaps are excluded: ownership moves without cloning.
    pub fn sorted_construction_storage_bytes(entry_count: usize) -> Option<usize> {
        storage_bounds::sorted_construction(entry_count)
    }

    /// Tree backing and contiguous decode staging for actual raw entry count.
    /// Nested heaps are excluded; duplicate keys may reduce final retention.
    pub fn decode_storage_bounds(entry_count: usize) -> Option<StructAspectValueStorageBounds> {
        storage_bounds::decode(entry_count)
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
        let mut value = Self {
            fields: super::field_tree::StructFields::new(),
            allocation_bytes: 0,
            semantic_bytes: 0,
        };
        for (key, field) in fields {
            if value.fields.contains_key(&key) {
                return Err(StructAspectValueConstructionDenial::DuplicateField(key));
            }
            value.insert(key.clone(), field);
        }
        Ok(value)
    }
    pub fn fields(&self) -> impl Iterator<Item = (&FieldKey, &AspectValue)> {
        self.fields.iter().map(|(key, value)| (key, value.as_ref()))
    }
    pub fn get(&self, key: &FieldKey) -> Option<&AspectValue> {
        self.fields.get(key).map(std::sync::Arc::as_ref)
    }
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.allocation_bytes.min(usize::MAX as u128) as usize
    }
    /// Checked reachable immutable backing for an independent retained holder.
    pub fn retained_storage_bytes(&self) -> Option<usize> {
        usize::try_from(self.allocation_bytes).ok()
    }
    pub(crate) fn semantic_byte_width(&self) -> usize {
        self.semantic_bytes.min(usize::MAX as u128) as usize
    }
    /// Cost: changed fields and persistent tree paths. Untouched values retain identity.
    pub(crate) fn patched<'a>(
        &self,
        sets: impl IntoIterator<Item = (&'a FieldKey, &'a AspectValue)>,
        clears: impl IntoIterator<Item = &'a FieldKey>,
    ) -> Self {
        let mut next = self.clone();
        for key in clears {
            next.remove(key);
        }
        for (key, value) in sets {
            next.insert(key.clone(), value.clone());
        }
        next
    }
    fn remove(&mut self, key: &FieldKey) {
        if let Some((key, value)) = self.fields.get_key_value(key) {
            let (allocation, semantic) = field_extent(key, value);
            self.allocation_bytes -= allocation;
            self.semantic_bytes -= semantic;
        }
        self.fields.remove(key);
    }
    fn insert(&mut self, key: FieldKey, value: AspectValue) {
        self.remove(&key);
        let (allocation, semantic) = field_extent(&key, &value);
        self.allocation_bytes += allocation;
        self.semantic_bytes += semantic;
        self.fields.insert(key, std::sync::Arc::new(value));
    }
}
fn field_extent(key: &FieldKey, value: &AspectValue) -> (u128, u128) {
    let allocation = super::field_tree::StructFields::allocation_bytes_per_field() as u128
        + key.owned_allocation_capacity_bytes() as u128
        + value.owned_allocation_capacity_bytes() as u128;
    (
        allocation,
        key.as_str().len() as u128 + value.semantic_byte_width() as u128,
    )
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
