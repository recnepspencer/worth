//! Inspection is a projection of the native codec's actual decoded input.
use crate::durability::data::*;
use crate::history::data::RecordAllocationClass;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
};

pub(crate) fn inspect<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &'a [u8],
    external: Option<std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
) -> Result<RelationalCheckpointInspection<'a>, RelationalRecoveryStop> {
    execution
        .check_live()
        .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
    let payload = super::checkpoint_payload(bytes)?;
    let (
        symbol_identity_frontier,
        record_identity,
        partition_images,
        branch_roots,
        branch_root_schema_images,
        branch_cells,
    ) = if let Some(decoded) = decode_for_inspection(execution, payload, external)? {
        (
            decoded.symbol_identity_frontier,
            decoded.record_identity,
            decoded.partition_images,
            decoded.branch_roots,
            decoded.branch_root_schema_images,
            decoded.branch_cells,
        )
    } else {
        let memory =
            crate::durability::checkpoint_history::inventory::prepare_decode(payload, execution)?;
        let mut wire: super::checkpoint_wire::CheckpointWire<
            '_,
            crate::durability::data::RelationalCheckpointManifestDraft,
        > = rmp_serde::from_slice(payload)
            .map_err(|error| invalid(format!("native manifest: {error}")))?;
        if let Err(memory) = wire.manifest.canonical_members.attach(memory) {
            drop(wire);
            drop(memory);
            return Err(invalid("canonical inventory shared before original admission").into());
        }
        crate::durability::checkpoint_history::inventory::validate(&wire.manifest, execution)?;
        let manifest = wire.manifest.seal_ordinary();
        let _ = super::expanded_checkpoint::validate_manifest(&manifest)?;
        let manifest = manifest.ordinary_owned_draft();
        let (_, branch_cells, _, _) = manifest.branch_data.ordinary_owned_parts();
        (
            manifest.symbol_identity_frontier,
            manifest.record_identity,
            Vec::new(),
            Vec::new(),
            manifest.schemas.ordinary_owned_copy(),
            branch_cells,
        )
    };
    if record_identity.schema_version != DurableRecordIdentityState::CURRENT_SCHEMA_VERSION {
        return Err(invalid("unknown record identity schema").into());
    }
    let frontiers = record_identity
        .append_frontiers
        .iter()
        .map(|row| {
            let DurableRecordSlotFrontier {
                class,
                partition_id,
                next_slot,
            } = row;
            CheckpointRecordFrontier {
                class: record_class(*class),
                partition_id: partition_id.0,
                next_slot: *next_slot,
            }
        })
        .collect();
    let generations = record_identity
        .generation_high_water
        .iter()
        .map(|row| {
            let DurableRecordGenerationHighWater {
                class,
                partition_id,
                slot,
                generation,
            } = row;
            CheckpointRecordIdentity {
                class: record_class(*class),
                partition_id: partition_id.0,
                slot: *slot,
                generation: *generation,
            }
        })
        .collect();
    let mut live_records = BTreeSet::new();
    let mut mirror_live_records = BTreeSet::new();
    let mut current_publication_records = BTreeSet::new();
    let mut roster_kinds = BTreeSet::new();
    for schema in &branch_root_schema_images {
        let (registry, _) = schema.readmit_registry().map_err(invalid)?;
        if let Some(storage) = registry.current_publication_storage {
            roster_kinds.extend(storage.metadata_kinds());
        }
    }
    // Alias targets are encoded in the mirror or another root. Union all
    // present arena images; a shared identity may be live in several roots.
    for (index, partitions) in std::iter::once(partition_images)
        .chain(branch_roots.into_iter().map(|root| root.partition_images))
        .enumerate()
    {
        let mut root_live = BTreeSet::new();
        for partition in partitions {
            census(
                RecordAllocationClass::Entity,
                partition.partition_id.0,
                partition.entity_arena,
                &mut root_live,
                &roster_kinds,
                &mut current_publication_records,
            )?;
            census(
                RecordAllocationClass::Relation,
                partition.partition_id.0,
                partition.relation_arena,
                &mut root_live,
                &roster_kinds,
                &mut current_publication_records,
            )?;
        }
        if index == 0 {
            mirror_live_records.extend(root_live.iter().copied());
        }
        live_records.extend(root_live);
    }
    let mut envelope = fields(payload)?;
    let mut fields = fields(take(&mut envelope, "manifest")?)?;
    take(&mut fields, "symbol_identity_frontier")?;
    let mut allocator = self::fields(take(&mut fields, "record_identity")?)?;
    take(&mut allocator, "append_frontiers")?;
    take(&mut allocator, "generation_high_water")?;
    Ok(RelationalCheckpointInspection {
        branch_cells,
        model: NativeCheckpointModelBytes {
            envelope,
            fields,
            allocator,
        },
        symbol_identity_frontier,
        frontiers,
        generations,
        mirror_live_records,
        live_records,
        current_publication_records,
    })
}

fn decode_for_inspection(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    payload: &[u8],
    external: Option<std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
) -> Result<Option<DurableCheckpoint>, RelationalRecoveryStop> {
    let Some(source) = external else {
        return Ok(None);
    };
    let memory =
        crate::durability::checkpoint_history::inventory::prepare_decode(payload, execution)?;
    let mut wire: super::checkpoint_wire::CheckpointWire<
        '_,
        crate::durability::data::RelationalCheckpointManifestDraft,
    > = rmp_serde::from_slice(payload)
        .map_err(|error| invalid(format!("native manifest: {error}")))?;
    if let Err(memory) = wire.manifest.canonical_members.attach(memory) {
        drop(wire);
        drop(memory);
        return Err(invalid("canonical inventory shared before original admission").into());
    }
    crate::durability::checkpoint_history::inventory::validate(&wire.manifest, execution)?;
    let manifest = wire.manifest.seal_ordinary();
    let facts = super::expanded_checkpoint::validate_manifest(&manifest)?;
    super::expanded_checkpoint::expand(execution, manifest, source, facts.history_members).map(Some)
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
    roster_kinds: &BTreeSet<crate::identity::data::KindId>,
    roster_records: &mut BTreeSet<CheckpointRecordIdentity>,
) -> Result<(), DurabilityError> {
    let RecordArenaCheckpointImage {
        slots,
        generations,
        lifecycle,
        kind_ids,
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
                let identity = CheckpointRecordIdentity {
                    class,
                    partition_id,
                    slot,
                    generation,
                };
                if kind_ids
                    .get(index)
                    .copied()
                    .flatten()
                    .is_some_and(|kind| roster_kinds.contains(&kind))
                {
                    roster_records.insert(identity);
                }
                if !result.insert(identity) {
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
