//! Declared names for presenting a Native uniqueness refusal.

use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::KindId;

use super::{WorthQueryPrimaryFieldLayout, WorthQueryPrimaryGraphLayout};

type FieldName = (String, String, String);

#[derive(Debug, Default)]
pub(super) struct WorthQueryUniqueFieldNames {
    by_kind: BTreeMap<KindId, BTreeMap<AspectFieldLocator, FieldName>>,
}

impl WorthQueryUniqueFieldNames {
    pub(super) fn lower(fields: &BTreeMap<FieldName, WorthQueryPrimaryFieldLayout>) -> Self {
        let mut by_kind = BTreeMap::<KindId, BTreeMap<AspectFieldLocator, FieldName>>::new();
        for (name, field) in fields.iter().filter(|(_, field)| field.unique) {
            by_kind
                .entry(field.entity_kind)
                .or_default()
                .insert(field.locator.clone(), name.clone());
        }
        Self { by_kind }
    }
}

impl WorthQueryPrimaryGraphLayout {
    /// The declared entity, aspect, and field names for an installed unique key.
    pub fn unique_field_name(
        &self,
        kind: KindId,
        locator: &AspectFieldLocator,
    ) -> Option<&(String, String, String)> {
        self.unique_fields.by_kind.get(&kind)?.get(locator)
    }
}
