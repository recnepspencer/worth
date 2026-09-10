use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{AspectKey, AspectValue, CanonicalFieldPath, StructAspectValue};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryGraphReadRow {
    entity_identity: Arc<str>,
    aspect_values: Vec<(AspectKey, AspectValue)>,
    struct_aspect_values: Vec<(AspectKey, StructAspectValue)>,
    field_values: Vec<(CanonicalFieldPath, AspectValue)>,
}

impl WorthQueryGraphReadRow {
    pub fn new(
        entity_identity: impl Into<Arc<str>>,
        aspect_values: BTreeMap<AspectKey, AspectValue>,
        struct_aspect_values: BTreeMap<AspectKey, StructAspectValue>,
        field_values: BTreeMap<CanonicalFieldPath, AspectValue>,
    ) -> Result<Self, WorthQueryGraphReadRowConstructionDenial> {
        let entity_identity = entity_identity.into();
        if entity_identity.trim().is_empty() {
            return Err(WorthQueryGraphReadRowConstructionDenial::EmptyEntityIdentity);
        }
        Ok(Self {
            entity_identity,
            aspect_values: aspect_values.into_iter().collect(),
            struct_aspect_values: struct_aspect_values.into_iter().collect(),
            field_values: field_values.into_iter().collect(),
        })
    }

    pub fn from_native_fields(
        entity_identity: impl Into<Arc<str>>,
        field_values: BTreeMap<CanonicalFieldPath, AspectValue>,
    ) -> Result<Self, WorthQueryGraphReadRowConstructionDenial> {
        Self::new(
            entity_identity,
            BTreeMap::new(),
            BTreeMap::new(),
            field_values,
        )
    }

    pub fn entity_identity(&self) -> &str {
        &self.entity_identity
    }

    pub fn aspect_value(&self, key: &AspectKey) -> Option<&AspectValue> {
        value_by_key(&self.aspect_values, key)
    }

    pub fn struct_aspect_value(&self, key: &AspectKey) -> Option<&StructAspectValue> {
        value_by_key(&self.struct_aspect_values, key)
    }

    pub fn field_value(&self, path: &CanonicalFieldPath) -> Option<&AspectValue> {
        value_by_key(&self.field_values, path)
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> usize {
        retained_arc_str_bytes(&self.entity_identity)
            .saturating_add(pair_vector_bytes(&self.aspect_values, |key, value| {
                key.owned_allocation_capacity_bytes()
                    .saturating_add(value.owned_allocation_capacity_bytes())
            }))
            .saturating_add(pair_vector_bytes(
                &self.struct_aspect_values,
                |key, value| {
                    key.owned_allocation_capacity_bytes()
                        .saturating_add(value.owned_allocation_capacity_bytes())
                },
            ))
            .saturating_add(pair_vector_bytes(&self.field_values, |path, value| {
                path.owned_allocation_capacity_bytes()
                    .saturating_add(value.owned_allocation_capacity_bytes())
            }))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadRowConstructionDenial {
    EmptyEntityIdentity,
}

fn value_by_key<'a, K: Ord, V>(values: &'a [(K, V)], key: &K) -> Option<&'a V> {
    values
        .binary_search_by(|(candidate, _)| candidate.cmp(key))
        .ok()
        .map(|index| &values[index].1)
}

fn pair_vector_bytes<K, V>(values: &Vec<(K, V)>, nested_bytes: impl Fn(&K, &V) -> usize) -> usize {
    values
        .capacity()
        .saturating_mul(std::mem::size_of::<(K, V)>())
        .saturating_add(
            values
                .iter()
                .map(|(key, value)| nested_bytes(key, value))
                .sum(),
        )
}

fn retained_arc_str_bytes(value: &Arc<str>) -> usize {
    value
        .len()
        .saturating_add(2_usize.saturating_mul(std::mem::size_of::<usize>()))
        .saturating_add(std::mem::align_of::<usize>().saturating_sub(1))
}
