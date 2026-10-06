use crate::durability::data::{DurabilityError, DurableCheckpoint, DurableRecordIdentityState};
use crate::history::data::RecordAllocationClass;
use crate::runtime::RecordIdentitySubsystem;

pub(super) fn prepare_record_identity(
    checkpoint: &DurableCheckpoint,
) -> Result<RecordIdentitySubsystem, DurabilityError> {
    let authority = RecordIdentitySubsystem::default();
    let durable = durable_record_identity(checkpoint)?;
    restore_record_generation_high_water(&authority, checkpoint, &durable.generation_high_water)?;
    restore_record_slot_allocator(&authority, checkpoint, durable)?;
    restore_pending_reservations(&authority, &durable.pending_reservations)?;
    Ok(authority)
}

fn restore_pending_reservations(
    authority: &RecordIdentitySubsystem,
    reservations: &[crate::durability::data::DurablePendingRecordReservation],
) -> Result<(), DurabilityError> {
    for reservation in reservations {
        let origin = match reservation.origin {
            crate::durability::data::DurableRecordReservationOrigin::AppendFrontier => {
                crate::history::data::RecordAllocationOrigin::AppendFrontier
            }
            crate::durability::data::DurableRecordReservationOrigin::Reclaimed {
                prior_generation,
            } => crate::history::data::RecordAllocationOrigin::Reclaimed { prior_generation },
        };
        authority
            .restore_pending(
                durable_record_class(reservation.class),
                reservation.partition_id,
                checked_slot(reservation.slot, "pending record reservation")?,
                reservation.generation,
                origin,
            )
            .map_err(|detail| corrupt_checkpoint(detail.to_owned()))?;
    }
    Ok(())
}

fn restore_record_generation_high_water(
    authority: &RecordIdentitySubsystem,
    checkpoint: &DurableCheckpoint,
    generation_high_water: &[crate::durability::data::DurableRecordGenerationHighWater],
) -> Result<(), DurabilityError> {
    for entry in generation_high_water {
        let slot = checked_slot(entry.slot, "record slot")?;
        authority.restore_generation(
            durable_record_class(entry.class),
            entry.partition_id,
            slot,
            entry.generation,
        );
    }
    for image in checkpoint.partition_images.iter().chain(
        checkpoint
            .branch_roots
            .iter()
            .flat_map(|root| root.partition_images.iter()),
    ) {
        observe_arena_generations(
            authority,
            RecordAllocationClass::Entity,
            image.partition_id,
            &image.entity_arena.slots,
            &image.entity_arena.generations,
        )?;
        observe_arena_generations(
            authority,
            RecordAllocationClass::Relation,
            image.partition_id,
            &image.relation_arena.slots,
            &image.relation_arena.generations,
        )?;
    }
    Ok(())
}

fn observe_arena_generations(
    authority: &RecordIdentitySubsystem,
    class: RecordAllocationClass,
    partition_id: crate::identity::data::PartitionId,
    slots: &[u64],
    generations: &[u32],
) -> Result<(), DurabilityError> {
    for (physical, &generation) in generations.iter().enumerate() {
        let logical = slots.get(physical).copied().unwrap_or(physical as u64);
        authority.restore_generation(
            class,
            partition_id,
            checked_slot(logical, "record slot")?,
            generation,
        );
    }
    Ok(())
}

fn restore_record_slot_allocator(
    authority: &RecordIdentitySubsystem,
    checkpoint: &DurableCheckpoint,
    identity: &DurableRecordIdentityState,
) -> Result<(), DurabilityError> {
    for image in checkpoint.partition_images.iter().chain(
        checkpoint
            .branch_roots
            .iter()
            .flat_map(|root| root.partition_images.iter()),
    ) {
        restore_arena_frontier(
            authority,
            RecordAllocationClass::Entity,
            image.partition_id,
            &image.entity_arena.slots,
            image.entity_arena.generations.len(),
        )?;
        restore_arena_frontier(
            authority,
            RecordAllocationClass::Relation,
            image.partition_id,
            &image.relation_arena.slots,
            image.relation_arena.generations.len(),
        )?;
    }
    for entry in &identity.append_frontiers {
        authority.restore_frontier(
            durable_record_class(entry.class),
            entry.partition_id,
            checked_slot(entry.next_slot, "record slot frontier")?,
        );
    }
    for entry in &identity.reusable_slots {
        authority.restore_reusable(
            durable_record_class(entry.class),
            entry.partition_id,
            checked_slot(entry.slot, "reusable record slot")?,
        );
    }
    Ok(())
}

/// The checkpoint's record identity state, refused unless this build wrote it.
fn durable_record_identity(
    checkpoint: &DurableCheckpoint,
) -> Result<&DurableRecordIdentityState, DurabilityError> {
    let identity = &checkpoint.record_identity;
    if identity.schema_version != DurableRecordIdentityState::CURRENT_SCHEMA_VERSION {
        return Err(corrupt_checkpoint(format!(
            "unsupported durable record identity schema version {}",
            identity.schema_version
        )));
    }
    Ok(identity)
}

fn restore_arena_frontier(
    authority: &RecordIdentitySubsystem,
    class: RecordAllocationClass,
    partition_id: crate::identity::data::PartitionId,
    slots: &[u64],
    row_count: usize,
) -> Result<(), DurabilityError> {
    let next_slot = match slots.iter().copied().max() {
        Some(slot) => slot
            .checked_add(1)
            .ok_or_else(|| corrupt_checkpoint("record slot frontier is exhausted".to_owned()))?,
        None => row_count as u64,
    };
    authority.restore_frontier(
        class,
        partition_id,
        checked_slot(next_slot, "record slot frontier")?,
    );
    Ok(())
}

fn checked_slot(slot: u64, label: &str) -> Result<usize, DurabilityError> {
    usize::try_from(slot).map_err(|_| {
        corrupt_checkpoint(format!(
            "{label} {slot} cannot be represented by this runtime"
        ))
    })
}

fn durable_record_class(
    class: crate::durability::data::DurableRecordGenerationClass,
) -> RecordAllocationClass {
    match class {
        crate::durability::data::DurableRecordGenerationClass::Entity => {
            RecordAllocationClass::Entity
        }
        crate::durability::data::DurableRecordGenerationClass::Relation => {
            RecordAllocationClass::Relation
        }
    }
}

fn corrupt_checkpoint(detail: String) -> DurabilityError {
    DurabilityError::new(
        crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
        detail,
    )
}
