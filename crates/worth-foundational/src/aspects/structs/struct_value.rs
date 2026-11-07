use std::collections::{BTreeMap, BTreeSet};

use super::{FieldDeclaration, FieldKey};
use crate::values::AspectValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructAspectShape {
    fields: Vec<FieldDeclaration>,
}

impl StructAspectShape {
    pub fn new(fields: impl IntoIterator<Item = FieldDeclaration>) -> Option<Self> {
        let mut fields: Vec<_> = fields.into_iter().collect();
        fields.sort_by(|left, right| left.key().cmp(right.key()));

        let mut seen = BTreeSet::new();
        for field in &fields {
            if !seen.insert(field.key().clone()) {
                return None;
            }
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
    fields: im::OrdMap<FieldKey, std::sync::Arc<AspectValue>>,
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
            fields: BTreeMap<FieldKey, AspectValue>,
        }
        Self::new(Image::deserialize(decoder)?.fields)
            .map_err(|error| serde::de::Error::custom(format!("{error:?}")))
    }
}
impl StructAspectValue {
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
            fields: im::OrdMap::new(),
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
    let allocation = std::mem::size_of::<(FieldKey, std::sync::Arc<AspectValue>)>() as u128
        + std::mem::size_of::<AspectValue>() as u128
        + (2 * std::mem::size_of::<usize>()) as u128
        + key.owned_allocation_capacity_bytes() as u128
        + value.owned_allocation_capacity_bytes() as u128;
    (
        allocation,
        key.as_str().len() as u128 + value.semantic_byte_width() as u128,
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructAspectValueConstructionDenial {
    DuplicateField(FieldKey),
}
