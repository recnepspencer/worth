use crate::visibility::materialization::read_records::{
    read_work::{record_memory, record_quote},
    KindScanVisibility,
};
use std::collections::BTreeMap;

use crate::runtime::RelationalRuntime;
use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::DenseSlotBitSet;

use super::pin_assembly::{assemble_snapshot_state_with_memory, SelectedRelationSlots};
use super::{HistoricalVisibilityBasis, SnapshotState, SnapshotStateBasis};

pub(crate) fn build_historical_visibility_state(
    runtime: &RelationalRuntime,
    basis: HistoricalVisibilityBasis,
    snapshot_id: SnapshotId,
    read_policy: SnapshotReadPolicy,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<SnapshotState, crate::execution::RelationalExecutionDenialCause> {
    debug_assert!(basis.source_version().as_u64() >= basis.version_id().as_u64());
    record_memory::claim(memory, basis.branch_id().0.len() as u64)?;
    let handle = SnapshotHandle {
        runtime_instance_id: runtime.runtime_instance_id(),
        branch_id: basis.branch_id().clone(),
        snapshot_id,
        version_id: basis.version_id(),
        read_policy,
    };
    if let Some(root) = basis.root().cloned() {
        return build_from_retained_state(runtime, basis, handle, root.as_ref(), charge, memory);
    }
    build_from_retained_state(runtime, basis, handle, &BTreeMap::new(), charge, memory)
}

fn build_from_retained_state(
    runtime: &RelationalRuntime,
    basis: HistoricalVisibilityBasis,
    handle: SnapshotHandle,
    state: &impl PartitionAccess,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<SnapshotState, crate::execution::RelationalExecutionDenialCause> {
    let version_id = basis.version_id();
    let registry = basis
        .root()
        .map_or(&runtime.config.schema.registry, |root| {
            root.schema_authority().registry()
        });
    let reader = runtime.read_truth();
    let entities = reader.visible_entity_slots_from_state(state, version_id, charge, memory)?;
    let visible = reader.visible_relation_slots_from_state(state, version_id, charge, memory)?;
    let visible_bytes = record_memory::vector_bytes::<(
        crate::identity::data::PartitionId,
        DenseSlotBitSet,
    )>(visible.capacity())?;
    let mut relations = Vec::new();
    for (partition_id, visible) in visible {
        let retained = retained_historical_relations(
            runtime,
            state,
            registry,
            partition_id,
            &visible,
            version_id,
            charge,
            memory,
        )?;
        record_memory::prepare_push(memory, &mut relations, 0)?;
        relations.push(SelectedRelationSlots {
            retained,
            partition_id,
            visible,
        });
    }
    record_memory::release(memory, visible_bytes)?;
    assemble_snapshot_state_with_memory(
        handle,
        SnapshotStateBasis::Historical(basis),
        entities,
        relations,
        memory,
    )
}

fn retained_historical_relations(
    runtime: &RelationalRuntime,
    state: &impl PartitionAccess,
    registry: &crate::schema::data::RelationalSchemaRegistry,
    partition_id: crate::identity::data::PartitionId,
    visible: &DenseSlotBitSet,
    version_id: crate::identity::data::VersionId,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<DenseSlotBitSet, crate::execution::RelationalExecutionDenialCause> {
    let reader = runtime.read_truth();
    let mut retained = DenseSlotBitSet::with_capacity(visible.represented_slot_capacity());
    let (slots, held) = record_memory::slots(visible, memory)?;
    for slot in slots {
        charge(crate::visibility::materialization::read_records::read_prices::KEYED_RECORD)
            .map_err(crate::execution::denial::kernel_stop)?;
        let relation_id = crate::identity::data::RelationId::new(partition_id, slot as u64, 0);
        let clone_bytes = record_quote::relation_for_id(
            state,
            registry,
            relation_id,
            KindScanVisibility::for_version(runtime, version_id),
        )?;
        record_memory::claim(memory, clone_bytes)?;
        let record = reader.authoritative_relation_record_for_id_at_version_with_registry(
            state,
            registry,
            relation_id,
            version_id,
        );
        let actual = record.as_ref().map_or(
            0,
            crate::visibility::materialization::read_records::read_footprint::relation,
        );
        record_memory::settle_clone(memory, clone_bytes, actual)?;
        let retain = record.is_some_and(|record| {
            record.lifecycle == crate::storage::data::RecordLifecycleState::RetainedDanglingForAudit
        });
        record_memory::release(memory, actual)?;
        if retain {
            record_memory::set_slot(memory, &mut retained, slot)?;
        }
    }
    record_memory::release(memory, held)?;
    Ok(retained)
}
