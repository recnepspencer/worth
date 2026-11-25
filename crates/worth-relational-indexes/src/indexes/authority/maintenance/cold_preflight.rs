//! Native-owned pre-intent budget admission for Home program transition target
//! cold index reconstruction.
//!
//! Mirrors the exact slot, live-bitset (including `RetainedDanglingForAudit`
//! relations), projection, and `CandidateIndexPreparation::prepare` per-index
//! work charges enforced by `changes.rs`, `cold_projection.rs`, and
//! `candidate_publication.rs`.

use std::collections::BTreeSet;
use std::sync::Arc;

use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;

use crate::contracts::execution::denial_cause::RelationalExecutionDenialCause;
use crate::contracts::identity::data::primitives::PartitionId;
use crate::contracts::indexes::data::definition::DerivedIndexKind;
use crate::contracts::indexes::data::maintenance::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};
use crate::runtime::state::subsystems::indexing::{
    IndexDefinitionCatalog, PreparedRelationalIndexDefinitions,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ColdReconstructionFootprint {
    pub entity_slot_count: usize,
    pub relation_slot_count: usize,
    pub region_count: usize,
    pub live_entity_slots: usize,
    pub live_relation_slots: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TargetCatalogIndexInventory {
    pub entity_field: usize,
    pub relation_field: usize,
    pub related_ordering: usize,
    pub relation_join: usize,
    pub total_indexes: usize,
}

impl TargetCatalogIndexInventory {
    pub fn from_catalog(catalog: &IndexDefinitionCatalog) -> Self {
        let mut inventory = Self::default();
        for index_id in catalog.index_ids() {
            let Some(definition) = catalog.definition(index_id) else {
                continue;
            };
            inventory.total_indexes = inventory.total_indexes.saturating_add(1);
            match &definition.kind {
                DerivedIndexKind::EntityField { .. } => {
                    inventory.entity_field = inventory.entity_field.saturating_add(1);
                }
                DerivedIndexKind::RelationField { .. } => {
                    inventory.relation_field = inventory.relation_field.saturating_add(1);
                }
                DerivedIndexKind::RelatedEntityOrdering { .. } => {
                    inventory.related_ordering = inventory.related_ordering.saturating_add(1);
                }
                DerivedIndexKind::RelationJoin(_) => {
                    inventory.relation_join = inventory.relation_join.saturating_add(1);
                }
            }
        }
        inventory
    }
}

/// Measure the exact predecessor root's slots and live-bitset record counts
/// (including `RetainedDanglingForAudit` relations via `live_slots()`), apply
/// target-step creation deltas, and admit `budget` against the prepared target
/// index catalog.
pub fn admit_home_target_cold_reconstruction_budget(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    predecessor_root: &Arc<RelationalBranchRoot>,
    definitions: Option<&PreparedRelationalIndexDefinitions>,
    created_entities: usize,
    created_relations: usize,
    touched_partitions: &[PartitionId],
    budget: DerivedIndexMaintenanceBudget,
) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
    let mut partitions: BTreeSet<PartitionId> = BTreeSet::new();
    let mut live_entity_slots = 0usize;
    let mut live_relation_slots = 0usize;

    for partition_id in predecessor_root.partition_ids_iter() {
        partitions.insert(partition_id);
        predecessor_root
            .demand_region_head_cached(execution, partition_id)
            .map_err(|failure| {
                use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;
                let cause = match failure {
                    Failure::StorageUnavailable(cause) => {
                        RelationalExecutionDenialCause::StorageUnavailable(cause)
                    }
                    Failure::Request(cause) => cause,
                    Failure::Content(cause) => cause.into_execution_cause(),
                };
                DerivedIndexMaintenanceDenial {
                    kind: Denial::Execution(cause),
                    work: DerivedIndexMaintenanceWork::default(),
                }
            })?;
        if let Some(head) = predecessor_root.accepted_head_region_view(partition_id) {
            live_entity_slots =
                live_entity_slots.saturating_add(head.entity_arena().live_slots().count());
            live_relation_slots =
                live_relation_slots.saturating_add(head.relation_arena().live_slots().count());
        }
    }
    for &partition_id in touched_partitions {
        partitions.insert(partition_id);
    }

    let footprint = ColdReconstructionFootprint {
        entity_slot_count: predecessor_root
            .entity_slot_count()
            .saturating_add(created_entities),
        relation_slot_count: predecessor_root
            .relation_slot_count()
            .saturating_add(created_relations),
        region_count: partitions.len(),
        live_entity_slots: live_entity_slots.saturating_add(created_entities),
        live_relation_slots: live_relation_slots.saturating_add(created_relations),
    };
    let inventory = definitions.map_or_else(TargetCatalogIndexInventory::default, |prepared| {
        TargetCatalogIndexInventory::from_catalog(&prepared.captured.catalog)
    });
    admit_footprint(footprint, inventory, budget)
}

/// Pure Native budget admission over exact cold-reconstruction footprint and
/// target catalog index inventory.
pub fn admit_footprint(
    footprint: ColdReconstructionFootprint,
    inventory: TargetCatalogIndexInventory,
    budget: DerivedIndexMaintenanceBudget,
) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
    let required_cold_slots = footprint
        .entity_slot_count
        .saturating_add(footprint.relation_slot_count)
        .saturating_add(footprint.region_count);

    if let Some(maximum) = budget.maximum_cold_record_slots {
        if maximum == 0 {
            return Err(DerivedIndexMaintenanceDenial {
                kind: Denial::ColdReconstructionRequired,
                work: DerivedIndexMaintenanceWork {
                    cold_record_slots: required_cold_slots,
                    ..DerivedIndexMaintenanceWork::default()
                },
            });
        }
        if required_cold_slots > maximum {
            return Err(DerivedIndexMaintenanceDenial {
                kind: Denial::ColdRecordSlotBudgetExceeded,
                work: DerivedIndexMaintenanceWork {
                    cold_record_slots: required_cold_slots,
                    ..DerivedIndexMaintenanceWork::default()
                },
            });
        }
    }

    // Derived-row upper bounds across all target index kinds (`cold_projection.rs`):
    // - EntityField: <= live_entity_slots rows per index
    // - RelationField / RelatedEntityOrdering: <= live_relation_slots rows per index
    // - RelationJoin: <= live_relation_slots^2 rows per index
    let max_entity_rows = inventory
        .entity_field
        .saturating_mul(footprint.live_entity_slots);
    let max_relation_rows = inventory
        .relation_field
        .saturating_add(inventory.related_ordering)
        .saturating_mul(footprint.live_relation_slots);
    let max_join_rows = inventory.relation_join.saturating_mul(
        footprint
            .live_relation_slots
            .saturating_mul(footprint.live_relation_slots),
    );
    let required_derived_rows = max_entity_rows
        .saturating_add(max_relation_rows)
        .saturating_add(max_join_rows);

    // Exact work-unit upper bound matching every `work.charge` / `work.read` /
    // `work.derive_row` call across `changes.rs`, `cold_projection.rs`,
    // `ordering.rs`, `join.rs`, and `candidate_publication.rs:126`:
    //   - changes.rs:65: `work.charge(slots)`
    //   - changes.rs:80,100: `work.read()` per live entity & relation bitset slot
    //     (including `RetainedDanglingForAudit` relations)
    //   - ordering.rs / join.rs: up to 3 `work.read()` per live relation per index
    //   - cold_projection.rs::push: `work.derive_row()` (1 unit per derived row)
    //   - candidate_publication.rs:126: `work.charge(prepared.entries.len())`
    let ordering_and_join_reads = inventory
        .related_ordering
        .saturating_add(inventory.relation_join)
        .saturating_mul(footprint.live_relation_slots.saturating_mul(3));
    let required_work_units = required_cold_slots
        .saturating_add(footprint.live_entity_slots)
        .saturating_add(footprint.live_relation_slots)
        .saturating_add(ordering_and_join_reads)
        .saturating_add(required_derived_rows)
        .saturating_add(inventory.total_indexes);

    let work = DerivedIndexMaintenanceWork {
        work_units: required_work_units,
        record_reads: footprint
            .live_entity_slots
            .saturating_add(footprint.live_relation_slots)
            .saturating_add(ordering_and_join_reads),
        cold_record_slots: required_cold_slots,
        derived_rows: required_derived_rows,
        generation_publications_reserved: inventory.total_indexes,
        ..DerivedIndexMaintenanceWork::default()
    };

    if budget
        .maximum_derived_rows
        .is_some_and(|maximum| required_derived_rows > maximum)
        || budget
            .maximum_work_units
            .is_some_and(|maximum| required_work_units > maximum)
    {
        return Err(DerivedIndexMaintenanceDenial {
            kind: Denial::WorkBudgetExceeded,
            work,
        });
    }

    Ok(work)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_preflight_charges_audit_retained_relations_and_per_index_publication_at_exact_boundary()
    {
        // 3 entity slots (2 live), 4 relation slots (2 live + 1 RetainedDanglingForAudit = 3 in live_bitset),
        // 1 region, and 2 EntityField + 1 RelatedEntityOrdering indexes (total 3 indexes).
        let footprint = ColdReconstructionFootprint {
            entity_slot_count: 3,
            relation_slot_count: 4,
            region_count: 1,
            live_entity_slots: 2,
            live_relation_slots: 3, // includes RetainedDanglingForAudit!
        };
        let inventory = TargetCatalogIndexInventory {
            entity_field: 2,
            relation_field: 0,
            related_ordering: 1,
            relation_join: 0,
            total_indexes: 3,
        };
        // slots = 3 + 4 + 1 = 8
        // derived_rows = (2 * 2) + (1 * 3) = 7
        // ordering_reads = 1 * (3 * 3) = 9
        // work_units = 8 (slots) + 2 (live_e) + 3 (live_r incl audit) + 9 (reads) + 7 (rows) + 3 (prepare entries) = 32
        let exact = DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(32),
            maximum_cold_record_slots: Some(8),
            maximum_derived_rows: Some(7),
        };
        let admitted = admit_footprint(footprint, inventory, exact).expect("exact budget admits");
        assert_eq!(admitted.cold_record_slots, 8);
        assert_eq!(admitted.derived_rows, 7);
        assert_eq!(admitted.work_units, 32);
        assert_eq!(admitted.generation_publications_reserved, 3);

        // N - 1 work units refuses with WorkBudgetExceeded.
        let one_short_work = DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(31),
            ..exact
        };
        assert_eq!(
            admit_footprint(footprint, inventory, one_short_work)
                .unwrap_err()
                .kind,
            Denial::WorkBudgetExceeded
        );

        // S - 1 cold slots refuses with typed ColdRecordSlotBudgetExceeded.
        let one_short_slots = DerivedIndexMaintenanceBudget {
            maximum_cold_record_slots: Some(7),
            ..exact
        };
        assert_eq!(
            admit_footprint(footprint, inventory, one_short_slots)
                .unwrap_err()
                .kind,
            Denial::ColdRecordSlotBudgetExceeded
        );

        // Zero cold slots means cold reconstruction is disallowed (structural).
        let zero_slots = DerivedIndexMaintenanceBudget {
            maximum_cold_record_slots: Some(0),
            ..exact
        };
        assert_eq!(
            admit_footprint(footprint, inventory, zero_slots)
                .unwrap_err()
                .kind,
            Denial::ColdReconstructionRequired
        );

        // R - 1 derived rows refuses with WorkBudgetExceeded.
        let one_short_rows = DerivedIndexMaintenanceBudget {
            maximum_derived_rows: Some(6),
            ..exact
        };
        assert_eq!(
            admit_footprint(footprint, inventory, one_short_rows)
                .unwrap_err()
                .kind,
            Denial::WorkBudgetExceeded
        );
    }
}
