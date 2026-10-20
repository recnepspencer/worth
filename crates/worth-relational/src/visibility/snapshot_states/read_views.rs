use crate::runtime::RelationalRuntime;
use crate::storage::data::{EntityReadRecord, RelationReadRecord, RelationalReadView};
use crate::storage::overlay::PartitionAccess;
use crate::visibility::materialization::read_records::{
    read_work::{record_memory, record_quote},
    KindScanVisibility,
};

use super::{SnapshotState, SnapshotStateBasis};

pub(crate) fn read_view_from_snapshot_state(
    runtime: &RelationalRuntime,
    state: &SnapshotState,
    handle: &crate::snapshots::data::SnapshotHandle,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<RelationalReadView, crate::execution::RelationalExecutionDenialCause> {
    debug_assert_eq!(state.basis.branch_id(), &handle.branch_id);
    debug_assert_eq!(state.basis.version_id(), handle.version_id);
    let (entities, relations) = match &state.basis {
        SnapshotStateBasis::Exact(basis) => {
            debug_assert!(match state.basis.root_version() {
                Some(root_version) => root_version == state.handle.version_id,
                None => state.handle.version_id.is_zero(),
            });
            materialize_exact(runtime, state, basis.root().as_ref(), charge, memory)?
        }
        SnapshotStateBasis::Historical(basis) => materialize_historical(
            runtime,
            state,
            basis.root(),
            state.handle.version_id,
            charge,
            memory,
        )?,
    };
    runtime.services.instrumentation.count(|counters| {
        counters.visible_authoritative_entity_records_materialized += entities.len();
        counters.visible_authoritative_relation_records_materialized += relations.len();
    });
    record_memory::claim(memory, handle.branch_id.0.len() as u64)?;
    Ok(RelationalReadView {
        snapshot: handle.clone(),
        entities,
        relations,
    })
}

fn materialize_exact(
    runtime: &RelationalRuntime,
    state: &SnapshotState,
    root: &crate::branch::RelationalBranchRoot,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<
    (Vec<EntityReadRecord>, Vec<RelationReadRecord>),
    crate::execution::RelationalExecutionDenialCause,
> {
    let reader = runtime.read_truth();
    let registry = root.schema_authority().registry();
    materialize_pins(
        state,
        |partition_id, slot| {
            reader.authoritative_entity_record_for_id_from_exact_state(
                root,
                registry,
                crate::identity::data::EntityId::new(partition_id, slot as u64, 0),
            )
        },
        |partition_id, slot| {
            reader.authoritative_relation_record_for_id_from_exact_state(
                root,
                registry,
                crate::identity::data::RelationId::new(partition_id, slot as u64, 0),
            )
        },
        |partition_id, slot| {
            record_quote::entity_for_id(
                root,
                registry,
                crate::identity::data::EntityId::new(partition_id, slot as u64, 0),
                KindScanVisibility::Live,
            )
        },
        |partition_id, slot| {
            record_quote::relation_for_id(
                root,
                registry,
                crate::identity::data::RelationId::new(partition_id, slot as u64, 0),
                KindScanVisibility::Live,
            )
        },
        charge,
        memory,
    )
}

fn materialize_historical(
    runtime: &RelationalRuntime,
    state: &SnapshotState,
    root: Option<&std::sync::Arc<crate::branch::RelationalBranchRoot>>,
    version_id: crate::identity::data::VersionId,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<
    (Vec<EntityReadRecord>, Vec<RelationReadRecord>),
    crate::execution::RelationalExecutionDenialCause,
> {
    let empty = std::collections::BTreeMap::new();
    let access: &dyn PartitionAccess = root.map_or(&empty, |root| root.as_ref());
    let registry = root.map_or(&runtime.config.schema.registry, |root| {
        root.schema_authority().registry()
    });
    let reader = runtime.read_truth();
    materialize_pins(
        state,
        |partition_id, slot| {
            reader.authoritative_entity_record_for_id_at_version_with_registry(
                access,
                registry,
                crate::identity::data::EntityId::new(partition_id, slot as u64, 0),
                version_id,
            )
        },
        |partition_id, slot| {
            reader.authoritative_relation_record_for_id_at_version_with_registry(
                access,
                registry,
                crate::identity::data::RelationId::new(partition_id, slot as u64, 0),
                version_id,
            )
        },
        |partition_id, slot| {
            record_quote::entity_for_id(
                access,
                registry,
                crate::identity::data::EntityId::new(partition_id, slot as u64, 0),
                KindScanVisibility::for_version(runtime, version_id),
            )
        },
        |partition_id, slot| {
            record_quote::relation_for_id(
                access,
                registry,
                crate::identity::data::RelationId::new(partition_id, slot as u64, 0),
                KindScanVisibility::for_version(runtime, version_id),
            )
        },
        charge,
        memory,
    )
}

fn materialize_pins(
    state: &SnapshotState,
    mut read_entity: impl FnMut(crate::identity::data::PartitionId, usize) -> Option<EntityReadRecord>,
    mut read_relation: impl FnMut(
        crate::identity::data::PartitionId,
        usize,
    ) -> Option<RelationReadRecord>,
    mut quote_entity: impl FnMut(
        crate::identity::data::PartitionId,
        usize,
    ) -> Result<u64, crate::execution::RelationalExecutionDenialCause>,
    mut quote_relation: impl FnMut(
        crate::identity::data::PartitionId,
        usize,
    ) -> Result<u64, crate::execution::RelationalExecutionDenialCause>,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<
    (Vec<EntityReadRecord>, Vec<RelationReadRecord>),
    crate::execution::RelationalExecutionDenialCause,
> {
    let vectors = record_memory::vector_bytes::<EntityReadRecord>(state.pinned_entity_count)?
        .checked_add(record_memory::vector_bytes::<RelationReadRecord>(
            state.pinned_relation_count,
        )?)
        .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)?;
    record_memory::claim(memory, vectors)?;
    let mut entities = Vec::with_capacity(state.pinned_entity_count);
    let mut relations = Vec::with_capacity(state.pinned_relation_count);
    for (partition_id, pins) in &state.pinned_partitions {
        let (slots, held) = record_memory::slots(&pins.entity_slots, memory)?;
        for slot in slots {
            charge(crate::visibility::materialization::read_records::read_prices::EXAMINE_SLOT)
                .map_err(crate::execution::denial::kernel_stop)?;
            charge(crate::visibility::materialization::read_records::read_prices::RESERVE_RECORD)
                .map_err(crate::execution::denial::kernel_stop)?;
            let clone_bytes = quote_entity(*partition_id, slot)?;
            record_memory::claim(memory, clone_bytes)?;
            if let Some(record) = read_entity(*partition_id, slot) {
                record_memory::settle_clone(
                    memory,
                    clone_bytes,
                    crate::visibility::materialization::read_records::read_footprint::entity(
                        &record,
                    ),
                )?;
                entities.push(record);
            } else {
                record_memory::release(memory, clone_bytes)?;
            }
        }
        record_memory::release(memory, held)?;
        let (slots, held) = record_memory::slots(&pins.relation_slots, memory)?;
        for slot in slots {
            charge(crate::visibility::materialization::read_records::read_prices::EXAMINE_SLOT)
                .map_err(crate::execution::denial::kernel_stop)?;
            charge(crate::visibility::materialization::read_records::read_prices::RESERVE_RECORD)
                .map_err(crate::execution::denial::kernel_stop)?;
            let clone_bytes = quote_relation(*partition_id, slot)?;
            record_memory::claim(memory, clone_bytes)?;
            if let Some(mut record) = read_relation(*partition_id, slot) {
                if pins
                    .retained_relation_slots
                    .count_ones_in_range(slot, slot + 1)
                    == 1
                {
                    record.lifecycle =
                        crate::storage::data::RecordLifecycleState::RetainedDanglingForAudit;
                }
                record_memory::settle_clone(
                    memory,
                    clone_bytes,
                    crate::visibility::materialization::read_records::read_footprint::relation(
                        &record,
                    ),
                )?;
                relations.push(record);
            } else {
                record_memory::release(memory, clone_bytes)?;
            }
        }
        record_memory::release(memory, held)?;
    }
    Ok((entities, relations))
}
