//! Inspection is a projection of the native codec's actual decoded input.
use super::*;
use crate::durability::data::*;
use crate::history::data::RecordAllocationClass;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
};

pub(crate) fn inspect<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &'a [u8],
) -> Result<RelationalCheckpointInspection<'a>, RelationalRecoveryStop> {
    execution
        .check_live()
        .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
    let payload = bytes
        .strip_prefix(crate::durability::log::native_file_codec::CHECKPOINT_MAGIC)
        .ok_or_else(|| invalid("unknown native checkpoint frame"))?;
    let wire: CheckpointFileWire<'_> = rmp_serde::from_slice(payload)
        .map_err(|e| invalid(format!("native checkpoint inspection: {e}")))?;
    // Exhaustive against the codec's actual type: a new field forces an owner
    // decision here. Every named model value below is compared as raw bytes.
    let CheckpointFileWire { checkpoint } = wire;
    let CheckpointWire {
        record_identity,
        partition_images,
        coverage: _,
        branch_cells: _,
        branch_roots,
        branch_root_schema_images: _,
        record_generation_high_water: _,
        reusable_record_slots: _,
        record_slot_frontiers: _,
        envelopes: _,
        aspect_contracts: _,
        lineage: _,
        index_definitions: _,
        derived_index_artifacts: _,
        derived_index_checkpoint: _,
        derived_index_checkpoint_format: _,
        symbol_table: _,
        runtime_name: _,
        partition_alias_format: _,
        branch_root_partition_aliases_v2: _,
    } = checkpoint;
    let DurableRecordIdentityState {
        schema_version,
        generation_high_water,
        append_frontiers,
        reusable_slots: _,
        pending_reservations: _,
    } = record_identity;
    if schema_version != DurableRecordIdentityState::CURRENT_SCHEMA_VERSION {
        return Err(invalid("unknown record identity schema").into());
    }
    let frontiers = append_frontiers
        .into_iter()
        .map(|row| {
            let DurableRecordSlotFrontier {
                class,
                partition_id,
                next_slot,
            } = row;
            CheckpointRecordFrontier {
                class: record_class(class),
                partition_id: partition_id.0,
                next_slot,
            }
        })
        .collect();
    let generations = generation_high_water
        .into_iter()
        .map(|row| {
            let DurableRecordGenerationHighWater {
                class,
                partition_id,
                slot,
                generation,
            } = row;
            CheckpointRecordIdentity {
                class: record_class(class),
                partition_id: partition_id.0,
                slot,
                generation,
            }
        })
        .collect();
    let mut live_records = BTreeSet::new();
    // Alias targets are encoded in the mirror or another root. Union all
    // present arena images; a shared identity may be live in several roots.
    for partitions in std::iter::once(partition_images)
        .chain(branch_roots.into_iter().map(|root| root.partition_images))
    {
        let mut root_live = BTreeSet::new();
        for partition in partitions {
            census(
                RecordAllocationClass::Entity,
                partition.partition_id.0,
                partition.entity_arena,
                &mut root_live,
            )?;
            census(
                RecordAllocationClass::Relation,
                partition.partition_id.0,
                partition.relation_arena,
                &mut root_live,
            )?;
        }
        live_records.extend(root_live);
    }
    let mut envelope = fields(payload)?;
    let mut fields = fields(take(&mut envelope, "checkpoint")?)?;
    let mut allocator = self::fields(take(&mut fields, "record_identity")?)?;
    take(&mut allocator, "append_frontiers")?;
    take(&mut allocator, "generation_high_water")?;
    Ok(RelationalCheckpointInspection {
        model: NativeCheckpointModelBytes {
            envelope,
            fields,
            allocator,
        },
        frontiers,
        generations,
        live_records,
    })
}
fn record_class(class: DurableRecordGenerationClass) -> RecordAllocationClass {
    match class {
        DurableRecordGenerationClass::Entity => RecordAllocationClass::Entity,
        DurableRecordGenerationClass::Relation => RecordAllocationClass::Relation,
    }
}
fn census<K: RecordArenaCheckpointKind>(
    class: RecordAllocationClass,
    partition_id: u32,
    arena: RecordArenaCheckpointImage<K>,
    result: &mut BTreeSet<CheckpointRecordIdentity>,
) -> Result<(), DurabilityError> {
    let RecordArenaCheckpointImage {
        slots,
        generations,
        lifecycle,
        kind_ids: _,
        metadata_history: _,
        created_at: _,
        retired_at: _,
        aspect_versions: _,
        field_revisions: _,
        extra: _,
        diagnostics_enrichment: _,
        branch_pins: _,
        replay_pins: _,
        snapshot_pins: _,
        live_bitset: _,
        reclaimable_bitset: _,
        free_list: _,
        marker: _,
    } = arena;
    if generations.len() != lifecycle.len()
        || (!slots.is_empty() && slots.len() != generations.len())
    {
        return Err(invalid("arena identity columns disagree"));
    }
    for (index, (&generation, lifecycle)) in generations.iter().zip(&lifecycle).enumerate() {
        use crate::storage::data::RecordLifecycleState as L;
        match lifecycle {
            L::Live | L::MaterializationUnavailable => {
                let slot = if slots.is_empty() {
                    index as u64
                } else {
                    slots[index]
                };
                if !result.insert(CheckpointRecordIdentity {
                    class,
                    partition_id,
                    slot,
                    generation,
                }) {
                    return Err(invalid("duplicate live record identity"));
                }
            }
            L::DeletedRetained
            | L::RetainedDanglingForAudit
            | L::PinnedBySnapshot
            | L::PinnedByBranch
            | L::PinnedByReplayRetention
            | L::Reclaimable
            | L::Reusable => {}
        }
    }
    Ok(())
}
fn invalid(message: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, message)
}
fn take<'a>(
    fields: &mut BTreeMap<String, &'a [u8]>,
    key: &str,
) -> Result<&'a [u8], DurabilityError> {
    fields
        .remove(key)
        .ok_or_else(|| invalid(format!("missing checkpoint field {key}")))
}
/// Generic MessagePack span reader; all grammar admission stays in the typed
/// native decoder above. Value bytes retain their original encoding.
fn fields(bytes: &[u8]) -> Result<BTreeMap<String, &[u8]>, DurabilityError> {
    let mut cursor = Cursor::new(bytes);
    let mut marker = [0];
    cursor
        .read_exact(&mut marker)
        .map_err(|e| invalid(e.to_string()))?;
    let count = match marker[0] {
        0x80..=0x8f => u32::from(marker[0] - 0x80),
        0xde => {
            let mut b = [0; 2];
            cursor
                .read_exact(&mut b)
                .map_err(|e| invalid(e.to_string()))?;
            u32::from(u16::from_be_bytes(b))
        }
        0xdf => {
            let mut b = [0; 4];
            cursor
                .read_exact(&mut b)
                .map_err(|e| invalid(e.to_string()))?;
            u32::from_be_bytes(b)
        }
        _ => return Err(invalid("checkpoint owner is not a map")),
    };
    let mut result = BTreeMap::new();
    for _ in 0..count {
        let key: String = rmp_serde::from_read(&mut cursor).map_err(|e| invalid(e.to_string()))?;
        let start = cursor.position() as usize;
        let _: serde::de::IgnoredAny =
            rmp_serde::from_read(&mut cursor).map_err(|e| invalid(e.to_string()))?;
        let end = cursor.position() as usize;
        if result.insert(key, &bytes[start..end]).is_some() {
            return Err(invalid("duplicate checkpoint field"));
        }
    }
    if cursor.position() as usize != bytes.len() {
        return Err(invalid("trailing checkpoint bytes"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
