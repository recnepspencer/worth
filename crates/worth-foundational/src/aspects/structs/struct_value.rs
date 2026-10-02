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
    fields: BTreeMap<FieldKey, AspectValue>,
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
            fields: canonical_fields,
        })
    }

    pub fn fields(&self) -> impl Iterator<Item = (&FieldKey, &AspectValue)> {
        self.fields.iter()
    }

    pub fn get(&self, key: &FieldKey) -> Option<&AspectValue> {
        self.fields.get(key)
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.fields
            .iter()
            .map(|(key, value)| {
                std::mem::size_of::<(FieldKey, AspectValue)>()
                    .saturating_add(key.owned_allocation_capacity_bytes())
                    .saturating_add(value.owned_allocation_capacity_bytes())
            })
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructAspectValueConstructionDenial {
    DuplicateField(FieldKey),
}
