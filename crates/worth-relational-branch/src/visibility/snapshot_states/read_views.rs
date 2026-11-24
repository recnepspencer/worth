use super::SnapshotRecordReadInputs;
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionAccess;
use crate::visibility::materialization::read_records::read_work::record_memory;
use crate::visibility::materialization::read_records::read_work::record_quote;
use crate::visibility::materialization::read_records::reader::kind_scan_visibility::KindScanVisibility;
use crate::visibility::snapshot_read_view::RelationalReadView;

use super::{SnapshotState, SnapshotStateBasis};

pub fn read_view_from_snapshot_state(
    inputs: SnapshotRecordReadInputs<'_>,
    state: &SnapshotState,
    handle: &crate::snapshots::data::SnapshotHandle,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<RelationalReadView, crate::execution::denial_cause::RelationalExecutionDenialCause> {
    debug_assert_eq!(
        /* Proof: The handle binding and state key originate from one admitted snapshot observation. The private cache lookup requires that exact key before this read view is constructed; neither field has a facade mutation door. */
        state.basis.branch_id(),
        &handle.branch_id
    );
    debug_assert_eq!(
        /* Proof: The handle binding and state key originate from one admitted snapshot observation. The private cache lookup requires that exact key before this read view is constructed; neither field has a facade mutation door. */
        state.basis.version_id(),
        handle.version_id
    );
    let (entities, relations) = match &state.basis {
        SnapshotStateBasis::Exact(basis) => {
            debug_assert!(
                /* Proof: The handle binding and state key originate from one admitted snapshot observation. The private cache lookup requires that exact key before this read view is constructed; neither field has a facade mutation door. */
                match state.basis.root_version() {
                    Some(root_version) => root_version == state.handle.version_id,
                    None => state.handle.version_id.is_zero(),
                }
            );
            let root = basis.projection_root();
            materialize_exact(
                inputs,
                state,
                &root,
                root.schema_authority().registry(),
                charge,
                memory,
            )?
        }
        SnapshotStateBasis::Historical(basis) => materialize_historical(
            inputs,
            state,
            basis
                .root()
                .map(|root| (root.partition_access(), root.schema_authority().registry())),
            state.handle.version_id,
            charge,
            memory,
        )?,
    };
    inputs.instrumentation().count(|counters| {
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
    inputs: SnapshotRecordReadInputs<'_>,
    state: &SnapshotState,
    storage: &impl PartitionAccess,
    registry: &RelationalSchemaRegistry,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<
    (Vec<EntityReadRecord>, Vec<RelationReadRecord>),
    crate::execution::denial_cause::RelationalExecutionDenialCause,
> {
    materialize_pins(
        state,
        |partition_id, slot| {
            inputs
                .reader()
                .authoritative_entity_record_for_id_from_exact_state(
                    storage,
                    registry,
                    crate::identity::data::EntityId::new(partition_id, slot as u64, 0),
                )
        },
        |partition_id, slot| {
            inputs
                .reader()
                .authoritative_relation_record_for_id_from_exact_state(
                    storage,
                    registry,
                    crate::identity::data::RelationId::new(partition_id, slot as u64, 0),
                )
        },
        |partition_id, slot| {
            record_quote::entity_for_id(
                storage,
                registry,
                crate::identity::data::EntityId::new(partition_id, slot as u64, 0),
                KindScanVisibility::Live,
            )
        },
        |partition_id, slot| {
            record_quote::relation_for_id(
                storage,
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
    inputs: SnapshotRecordReadInputs<'_>,
    state: &SnapshotState,
    selected: Option<(&dyn PartitionAccess, &RelationalSchemaRegistry)>,
    version_id: crate::identity::data::VersionId,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<
    (Vec<EntityReadRecord>, Vec<RelationReadRecord>),
    crate::execution::denial_cause::RelationalExecutionDenialCause,
> {
    let empty = std::collections::BTreeMap::new();
    let (access, registry) = selected.unwrap_or((&empty, inputs.fallback_schema()));

    materialize_pins(
        state,
        |partition_id, slot| {
            inputs
                .reader()
                .authoritative_entity_record_for_id_at_version_with_registry(
                    access,
                    registry,
                    crate::identity::data::EntityId::new(partition_id, slot as u64, 0),
                    version_id,
                )
        },
        |partition_id, slot| {
            inputs
                .reader()
                .authoritative_relation_record_for_id_at_version_with_registry(
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
                KindScanVisibility::for_selected_version(inputs.current_version(), version_id),
            )
        },
        |partition_id, slot| {
            record_quote::relation_for_id(
                access,
                registry,
                crate::identity::data::RelationId::new(partition_id, slot as u64, 0),
                KindScanVisibility::for_selected_version(inputs.current_version(), version_id),
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
    ) -> Result<
        u64,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >,
    mut quote_relation: impl FnMut(
        crate::identity::data::PartitionId,
        usize,
    ) -> Result<
        u64,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<
    (Vec<EntityReadRecord>, Vec<RelationReadRecord>),
    crate::execution::denial_cause::RelationalExecutionDenialCause,
> {
    let vectors = record_memory::vector_bytes::<EntityReadRecord>(state.pinned_entity_count)?
        .checked_add(record_memory::vector_bytes::<RelationReadRecord>(
            state.pinned_relation_count,
        )?)
        .ok_or(
            crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
        )?;
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
