use std::collections::BTreeMap;

use crate::runtime::RelationalRuntime;
use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::DenseSlotBitSet;

use super::pin_assembly::{assemble_snapshot_state, SelectedRelationSlots};
use super::{HistoricalVisibilityBasis, SnapshotState, SnapshotStateBasis};

pub(crate) fn build_historical_visibility_state(
    runtime: &RelationalRuntime,
    basis: HistoricalVisibilityBasis,
    snapshot_id: SnapshotId,
    read_policy: SnapshotReadPolicy,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
) -> Result<SnapshotState, worth_execution::MapKernelStop> {
    debug_assert!(basis.source_version().as_u64() >= basis.version_id().as_u64());
    let handle = SnapshotHandle {
        runtime_instance_id: runtime.runtime_instance_id(),
        branch_id: basis.branch_id().clone(),
        snapshot_id,
        version_id: basis.version_id(),
        read_policy,
    };
    if let Some(root) = basis.root().cloned() {
        return build_from_retained_state(runtime, basis, handle, root.as_ref(), charge);
    }
    build_from_retained_state(runtime, basis, handle, &BTreeMap::new(), charge)
}

fn build_from_retained_state(
    runtime: &RelationalRuntime,
    basis: HistoricalVisibilityBasis,
    handle: SnapshotHandle,
    state: &impl PartitionAccess,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
) -> Result<SnapshotState, worth_execution::MapKernelStop> {
    let version_id = basis.version_id();
    let registry = basis
        .root()
        .map_or(&runtime.config.schema.registry, |root| {
            root.schema_authority().registry()
        });
    let reader = runtime.read_truth();
    let entities = reader.visible_entity_slots_from_state(state, version_id, charge)?;
    let relations = reader
        .visible_relation_slots_from_state(state, version_id, charge)?
        .into_iter()
        .map(|(partition_id, visible)| {
            let retained = retained_historical_relations(
                runtime,
                state,
                registry,
                partition_id,
                &visible,
                version_id,
                charge,
            )?;
            Ok(SelectedRelationSlots {
                retained,
                partition_id,
                visible,
            })
        })
        .collect::<Result<Vec<_>, worth_execution::MapKernelStop>>()?;
    Ok(assemble_snapshot_state(
        handle,
        SnapshotStateBasis::Historical(basis),
        entities,
        relations,
    ))
}

fn retained_historical_relations(
    runtime: &RelationalRuntime,
    state: &impl PartitionAccess,
    registry: &crate::schema::data::RelationalSchemaRegistry,
    partition_id: crate::identity::data::PartitionId,
    visible: &DenseSlotBitSet,
    version_id: crate::identity::data::VersionId,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
) -> Result<DenseSlotBitSet, worth_execution::MapKernelStop> {
    let reader = runtime.read_truth();
    let mut retained = DenseSlotBitSet::with_capacity(visible.represented_slot_capacity());
    for slot in visible.set_slots() {
        charge(crate::visibility::materialization::read_records::read_prices::KEYED_RECORD)?;
        let relation_id = crate::identity::data::RelationId::new(partition_id, slot as u64, 0);
        let record = reader.authoritative_relation_record_for_id_at_version_with_registry(
            state,
            registry,
            relation_id,
            version_id,
        );
        if record.is_some_and(|record| {
            record.lifecycle == crate::storage::data::RecordLifecycleState::RetainedDanglingForAudit
        }) {
            retained.set(slot, true);
        }
    }
    Ok(retained)
}
