//! One paid occupied-slot catalog scan; no record vector or per-hop ID lookup.
use super::super::{cohorts::StoredAggregate, funding::AggregateWork};
use super::*;
use crate::branch::{RelationalBranchRoot, RelationalPartitionVisit};
use crate::schema::data::AggregateIdentityContractDeclaration;
use crate::storage::substrate::RecordSlotVisit;
use worth_foundational::facade::AggregateIdentity;

pub(super) fn find(
    root: &RelationalBranchRoot,
    view: &InvariantStateView<'_>,
    contract: AggregateIdentityContractDeclaration,
    identity: AggregateIdentity,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<StoredAggregate> {
    // A scalar selection summary stays on the stack, not in a retained catalog.
    let mut found = None;
    root.try_visit_partitions(|event| {
        work.checkpoint(1)?;
        let RelationalPartitionVisit::Partition(partition_id) = event else {
            return Ok(());
        };
        let Some(partition) = view.state().get_partition(partition_id) else {
            return work.invalid(PlanDenial::InvalidStoredIdentity);
        };
        partition.entity_arena.try_visit_occupied(|event| {
            match event {
                RecordSlotVisit::Navigation(units) => {
                    work.checkpoint(units)?;
                }
                RecordSlotVisit::Slot {
                    logical,
                    view: slot,
                } => {
                    work.checkpoint(1)?;
                    if slot.is_materialization_unavailable()
                        && slot.kind_id() == Some(contract.storage.aggregate_kind)
                    {
                        return work.invalid(PlanDenial::InvalidStoredIdentity);
                    }
                    if !slot.is_live() {
                        return Ok(());
                    }
                    let Some(kind) = slot.kind_id() else {
                        return work.invalid(PlanDenial::InvalidStoredIdentity);
                    };
                    work.acquired(Acquisition::Entity)?;
                    if kind != contract.storage.aggregate_kind {
                        return Ok(());
                    }
                    if slot.partition_id() != partition_id {
                        return work.invalid(PlanDenial::InvalidStoredIdentity);
                    }
                    let entity = EntityId::new(partition_id, logical as u64, slot.generation());
                    let row = super::super::stored_record::decode(view, entity, kind, work)?;
                    if row.definition == contract.definition && row.identity == identity {
                        if found
                            .replace(StoredAggregate {
                                entity,
                                record: row,
                            })
                            .is_some()
                        {
                            return work.invalid(PlanDenial::AmbiguousIdentity);
                        }
                    }
                }
            }
            Ok(())
        })
    })?;
    found.ok_or(MapKernelFailure::Domain(PlanDenial::IdentityUnavailable))
}
