//! Debits each admitted C.9 projection from the bounded decode allowance.

use worth_store_physical_format::{
    PersistedPhysicalRecoveryProjection, PhysicalRecoveryProjectionDecodeLimits,
};

use super::super::PhysicalRedoPlanningDenial;

pub(super) fn consume_projection_limits(
    remaining: &mut PhysicalRecoveryProjectionDecodeLimits,
    projection: &PersistedPhysicalRecoveryProjection,
) -> Result<(), PhysicalRedoPlanningDenial> {
    remaining.frames = remaining
        .frames
        .checked_sub(match projection.payload() {
            worth_store_physical_format::PersistedPhysicalRecoveryPayload::Frames(frames) => {
                frames.len() as u64
            }
            worth_store_physical_format::PersistedPhysicalRecoveryPayload::SourceCopy(_) => 0,
        })
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    remaining.record_identities = remaining
        .record_identities
        .checked_sub(projection.record_identities().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    remaining.placements = remaining
        .placements
        .checked_sub(projection.placements().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    remaining.segment_updates = remaining
        .segment_updates
        .checked_sub(projection.segment_updates().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    remaining.manifests = remaining
        .manifests
        .checked_sub(projection.manifests().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    let consumed_entries = projection
        .placements()
        .len()
        .saturating_add(projection.segment_updates().len())
        .saturating_add(projection.manifests().len()) as u64;
    let consumed_entries = consumed_entries
        .checked_add(
            (match projection.operation() {
                worth_store_physical_format::PersistedPhysicalRecoveryOperation::RecordsDropped {
                    head_effect,
                    ..
                } => head_effect.as_ref(),
                _ => None,
            })
                .map(|effect| {
                    effect
                        .entry_count()
                        .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)
                })
                .transpose()?
                .unwrap_or(0),
        )
        .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
    remaining.total_entries = remaining
        .total_entries
        .checked_sub(consumed_entries)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    remaining.inline_allocations = remaining
        .inline_allocations
        .checked_sub(projection.root_state().inline_allocations().len() as u64)
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    Ok(())
}
