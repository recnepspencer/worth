use std::collections::BTreeMap;

use worth_foundational::facade::{AspectFieldLocator, AspectKey, AspectValue};

use crate::identity::data::{EntityId, KindId};
use crate::storage::data::authoritative_field_comparison_key::{
    authoritative_aspect_value_field_comparison_key, AuthoritativeFieldComparisonKey,
};
use crate::visibility::materialization::read_records::projection::projection_records::EntityProjectionRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyProjectedAspectReadSet {
    projected_values: BTreeMap<AspectKey, AspectValue>,
}

impl StrategyProjectedAspectReadSet {
    pub fn projected_aspect_value(&self, aspect_key: &AspectKey) -> Option<&AspectValue> {
        self.projected_values.get(aspect_key)
    }

    pub fn projected_field_comparison_key(
        &self,
        field_locator: &AspectFieldLocator,
    ) -> Option<AuthoritativeFieldComparisonKey> {
        self.projected_aspect_value(field_locator.aspect().aspect_key())
            .map(authoritative_aspect_value_field_comparison_key)
    }

    fn from_projection(record: EntityProjectionRecord<'_>, aspect_keys: &[AspectKey]) -> Self {
        let projected_values = aspect_keys
            .iter()
            .filter_map(|aspect_key| {
                record
                    .aspect_value(aspect_key)
                    .cloned()
                    .map(|value| (aspect_key.clone(), value))
            })
            .collect();
        Self { projected_values }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyEntityAspectReadRecord {
    entity_id: EntityId,
    kind_id: KindId,
    kind_name: String,
    projected_aspect_reads: StrategyProjectedAspectReadSet,
}

impl StrategyEntityAspectReadRecord {
    pub(crate) fn from_projection(
        record: EntityProjectionRecord<'_>,
        aspect_keys: &[AspectKey],
    ) -> Self {
        let projected_aspect_reads =
            StrategyProjectedAspectReadSet::from_projection(record, aspect_keys);
        Self {
            entity_id: record.entity_id(),
            kind_id: record.kind_id(),
            kind_name: record.kind_name().to_owned(),
            projected_aspect_reads,
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn kind_id(&self) -> KindId {
        self.kind_id
    }

    pub fn kind_name(&self) -> &str {
        &self.kind_name
    }

    pub fn projected_aspect_reads(&self) -> &StrategyProjectedAspectReadSet {
        &self.projected_aspect_reads
    }

    pub fn projected_field_comparison_key(
        &self,
        field_locator: &AspectFieldLocator,
    ) -> Option<AuthoritativeFieldComparisonKey> {
        self.projected_aspect_reads
            .projected_field_comparison_key(field_locator)
    }
}
