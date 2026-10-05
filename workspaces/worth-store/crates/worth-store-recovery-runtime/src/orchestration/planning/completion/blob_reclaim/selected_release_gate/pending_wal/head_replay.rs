//! Admit the exact C.9 effect against actual selected head-path media.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_recovery_physics::{
    HeadReplayBound, ImmutablePhysicalRedoPlan, PhysicalRedoProjection, PhysicalSourceSelection,
    SelectedReleaseHeadReplayDenial, VerifiedSelectedReleaseHeadReplayV14,
};

use crate::orchestration::planning::completion::historical_publication::{
    discovery_failure, HistoricalFailure,
};
use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;

pub(super) fn admit(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    budget: &mut ManifestEntryBudget,
    projection: &PhysicalRedoProjection,
    redo: &ImmutablePhysicalRedoPlan,
    selected: &PhysicalSourceSelection,
    staging_bytes: u64,
    resident: &mut ResidentAllowance,
) -> Result<VerifiedSelectedReleaseHeadReplayV14, HistoricalFailure> {
    const INVALID: HistoricalFailure = HistoricalFailure::Invalid;
    let worth_store_physical_format::PersistedPhysicalRecoveryOperation::RecordsDropped {
        head_effect: Some(effect),
        ..
    } = projection.materialization().operation()
    else {
        return Err(INVALID);
    };
    let format = selected.root().selected().selector().format();
    let peak = effect
        .verification_additional_peak_bytes(format)
        .zip(u64::from(format.page_size().bytes()).checked_mul(2))
        .zip(effect.owned_heap_bytes())
        .map(|((verification, pages), owned)| verification.max(pages).max(owned))
        .ok_or(INVALID)?;
    resident.transient(peak).map_err(|_| INVALID)?;
    // Replaying one member's head path is one lookup, however many blocks
    // the path crosses.
    budget.consume(1)?;
    // Physics carries no reason across its reader, so the reader keeps it.
    let mut unread = None;
    let replay = VerifiedSelectedReleaseHeadReplayV14::admit_projection(
        projection,
        redo,
        selected,
        staging_bytes,
        resident.remaining(),
        |reference, maximum| {
            discovery
                .read_release_custody_head_block(reference.generation(), reference.block(), maximum)
                .map_err(|failure| unread = Some(discovery_failure(failure)))?
                .bytes()
                .map(<[u8]>::to_vec)
                .ok_or(())
        },
    )
    .map_err(|denial| unread.unwrap_or_else(|| refused(denial, resident)))?;
    resident
        .bytes(replay.owned_heap_bytes().ok_or(INVALID)?)
        .map_err(|_| INVALID)?;
    Ok(replay)
}

/// What physics refused. The effect bytes it was admitted are recovery's
/// staging bytes; the heap is what the allowance had left, which keeps the
/// size it could not hold for the gate to report.
fn refused(
    denial: SelectedReleaseHeadReplayDenial,
    resident: &mut ResidentAllowance,
) -> HistoricalFailure {
    let SelectedReleaseHeadReplayDenial::BoundExceeded(past) = denial else {
        return HistoricalFailure::Invalid;
    };
    match past.bound {
        HeadReplayBound::EffectBytes => HistoricalFailure::StagingBytes(past.observed),
        HeadReplayBound::HeapBytes => {
            let _ = resident.transient(past.observed);
            HistoricalFailure::Invalid
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_store_recovery_physics::ExceededHeadReplayBound;

    use super::*;

    #[test]
    fn a_bound_physics_refused_is_the_limit_that_ran_out_and_not_damage() {
        let past = |bound, observed, admitted| {
            SelectedReleaseHeadReplayDenial::BoundExceeded(ExceededHeadReplayBound {
                bound,
                observed,
                admitted,
            })
        };
        let mut resident = ResidentAllowance::new(64);
        resident.bytes(24).unwrap();
        assert_eq!(
            refused(past(HeadReplayBound::EffectBytes, 897, 896), &mut resident),
            HistoricalFailure::StagingBytes(897)
        );
        assert_eq!(
            refused(SelectedReleaseHeadReplayDenial::SourcePath, &mut resident),
            HistoricalFailure::Invalid
        );
        assert_eq!(resident.exceeded_requirement(), None);
        // Forty bytes were left and the replay needed 41 at once.
        assert_eq!(
            refused(past(HeadReplayBound::HeapBytes, 41, 40), &mut resident),
            HistoricalFailure::Invalid
        );
        assert_eq!(resident.exceeded_requirement(), Some(65));
    }
}
