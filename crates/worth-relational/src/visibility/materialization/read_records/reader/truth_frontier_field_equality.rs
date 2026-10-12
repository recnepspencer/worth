use std::collections::BTreeSet;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use super::*;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::materialization::read_records::entity_query_locus_comparison_key;

#[derive(Debug)]
pub struct BoundedFrontierFieldEqualityTruthRead {
    matching_entity_ids: BTreeSet<crate::identity::data::EntityId>,
    entity_records_examined: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontierFieldEqualityTruthReadLimitExceeded {
    entity_records_examined: usize,
    matching_entity_ids_reserved: usize,
}

impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn bounded_entity_field_equals_for_frontier_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<crate::identity::data::EntityId>,
        kind_id: crate::identity::data::KindId,
        field_locator: &AspectFieldLocator,
        expected: &AspectValue,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedFrontierFieldEqualityTruthRead, FrontierFieldEqualityTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        crate::visibility::materialization::read_records::read_request::checked(execution, || {
            let expected = AuthoritativeFieldComparisonKey::from_aspect_value(expected);
            let state = self.runtime.storage_access().current_edition();
            let mut matching_entity_ids = BTreeSet::new();
            let mut entity_records_examined = 0_usize;
            for entity_id in entity_ids {
                charge_frontier_field_work(
                    maximum_work_units,
                    entity_records_examined,
                    matching_entity_ids.len(),
                )?;
                entity_records_examined += 1;
                let Some(record) = self
                    .authoritative_entity_record_for_id_at_version(&state, *entity_id, version_id)
                else {
                    continue;
                };
                if record.kind.kind_id != kind_id
                    || record.lifecycle != crate::storage::data::RecordLifecycleState::Live
                    || entity_query_locus_comparison_key(&record, field_locator).as_ref()
                        != Some(&expected)
                {
                    continue;
                }
                charge_frontier_field_work(
                    maximum_work_units,
                    entity_records_examined,
                    matching_entity_ids.len(),
                )?;
                matching_entity_ids.insert(record.entity_id);
            }
            Ok(BoundedFrontierFieldEqualityTruthRead {
                matching_entity_ids,
                entity_records_examined,
            })
        })
    }
}

impl BoundedFrontierFieldEqualityTruthRead {
    pub const fn entity_records_examined(&self) -> usize {
        self.entity_records_examined
    }

    pub fn matching_entity_ids(&self) -> &BTreeSet<crate::identity::data::EntityId> {
        &self.matching_entity_ids
    }

    pub fn matching_entity_ids_reserved(&self) -> usize {
        self.matching_entity_ids.len()
    }

    pub fn work_units(&self) -> usize {
        self.entity_records_examined
            .saturating_add(self.matching_entity_ids.len())
    }

    pub fn into_matching_entity_ids(self) -> BTreeSet<crate::identity::data::EntityId> {
        self.matching_entity_ids
    }
}

impl FrontierFieldEqualityTruthReadLimitExceeded {
    const fn new(entity_records_examined: usize, matching_entity_ids_reserved: usize) -> Self {
        Self {
            entity_records_examined,
            matching_entity_ids_reserved,
        }
    }

    pub const fn entity_records_examined(self) -> usize {
        self.entity_records_examined
    }

    pub const fn matching_entity_ids_reserved(self) -> usize {
        self.matching_entity_ids_reserved
    }

    pub const fn consumed_work_units(self) -> usize {
        self.entity_records_examined
            .saturating_add(self.matching_entity_ids_reserved)
    }
}

fn charge_frontier_field_work(
    maximum_work_units: usize,
    entity_records_examined: usize,
    matching_entity_ids_reserved: usize,
) -> Result<(), FrontierFieldEqualityTruthReadLimitExceeded> {
    if entity_records_examined.saturating_add(matching_entity_ids_reserved) >= maximum_work_units {
        Err(FrontierFieldEqualityTruthReadLimitExceeded::new(
            entity_records_examined,
            matching_entity_ids_reserved,
        ))
    } else {
        Ok(())
    }
}

impl ProjectionRecordReader<'_> {
    pub(crate) fn bounded_entity_field_equals_for_frontier_in_state<E>(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        mut check_live: impl FnMut() -> Result<(), E>,
        entity_ids: &BTreeSet<crate::identity::data::EntityId>,
        kind_id: crate::identity::data::KindId,
        field_locator: &AspectFieldLocator,
        expected: &AspectValue,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        BoundedFrontierFieldEqualityTruthRead,
        BoundedFrontierReadStop<FrontierFieldEqualityTruthReadLimitExceeded, E>,
    > {
        let expected = AuthoritativeFieldComparisonKey::from_aspect_value(expected);
        let mut matching_entity_ids = BTreeSet::new();
        let mut entity_records_examined = 0_usize;
        for entity_id in entity_ids {
            charge_frontier_field_work(
                maximum_work_units,
                entity_records_examined,
                matching_entity_ids.len(),
            )
            .map_err(BoundedFrontierReadStop::Limit)?;
            check_live().map_err(BoundedFrontierReadStop::Admission)?;
            entity_records_examined += 1;
            let Some(record) = self.authoritative_entity_record_for_id_at_version_with_registry(
                state, registry, *entity_id, version_id,
            ) else {
                continue;
            };
            if record.kind.kind_id != kind_id
                || record.lifecycle != crate::storage::data::RecordLifecycleState::Live
                || entity_query_locus_comparison_key(&record, field_locator).as_ref()
                    != Some(&expected)
            {
                continue;
            }
            charge_frontier_field_work(
                maximum_work_units,
                entity_records_examined,
                matching_entity_ids.len(),
            )
            .map_err(BoundedFrontierReadStop::Limit)?;
            check_live().map_err(BoundedFrontierReadStop::Admission)?;
            matching_entity_ids.insert(record.entity_id);
        }
        Ok(BoundedFrontierFieldEqualityTruthRead {
            matching_entity_ids,
            entity_records_examined,
        })
    }
}
