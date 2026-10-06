//! Admit the exact C.9 effect against actual selected head-path media.

use worth_store::physical_runtime::{
    ArtifactCeiling, BoundedRecoveryFilesystemDiscovery, PageAddress, ReadGrant, UnchargedRead,
};
use worth_store_recovery_physics::{
    HeadReplayBound, ImmutablePhysicalRedoPlan, PhysicalRedoProjection, PhysicalSourceSelection,
    SelectedReleaseHeadReplayDenial, VerifiedSelectedReleaseHeadReplayV14,
};

use crate::entry::{PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension};
use crate::orchestration::planning::completion::historical_publication::{
    discovery_failure, HistoricalFailure,
};
use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;
use crate::orchestration::planning::page_observation::PageLimit;
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;
use crate::orchestration::recovery_budget::RecoveryAllowance;

pub(super) fn admit(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    budget: &mut ManifestEntryBudget,
    projection: &PhysicalRedoProjection,
    redo: &ImmutablePhysicalRedoPlan,
    selected: &PhysicalSourceSelection,
    limits: &PhysicalRecoveryLimitDeclaration,
    resident: &mut ResidentAllowance,
) -> Result<VerifiedSelectedReleaseHeadReplayV14, HistoricalFailure> {
    const INVALID: HistoricalFailure = HistoricalFailure::Invalid;
    let staging = RecoveryAllowance::declared(limits, PhysicalRecoveryLimitDimension::StagingBytes);
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
        staging.admitted(),
        resident.remaining(),
        // Each block is one page; physics passes a page as its maximum.
        |reference, _page| {
            let address = PageAddress::ReleaseCustodyHeadBlock {
                generation: reference.generation(),
                block: reference.block(),
            };
            discovery
                .read(
                    ArtifactCeiling::page(format, address),
                    ReadGrant::ceiling_only(),
                )
                .observed()
                .map_err(|failure| unread = Some(discovery_failure(failure)))?
                .bytes()
                .map(<[u8]>::to_vec)
                .ok_or(())
        },
    )
    .map_err(|denial| unread.unwrap_or_else(|| refused(denial, staging, resident)))?;
    resident
        .bytes(replay.owned_heap_bytes().ok_or(INVALID)?)
        .map_err(|_| INVALID)?;
    Ok(replay)
}

/// What physics refused. The effect bytes it was admitted are all of
/// recovery's `staging`, held beside nothing else; the heap is what the
/// allowance had left, which keeps the size it could not hold for the gate to
/// report.
fn refused(
    denial: SelectedReleaseHeadReplayDenial,
    staging: RecoveryAllowance,
    resident: &mut ResidentAllowance,
) -> HistoricalFailure {
    use SelectedReleaseHeadReplayDenial as Denial;
    let past = match denial {
        Denial::BoundExceeded(past) => past,
        Denial::SizeOverflow => return HistoricalFailure::CountOverflow,
        Denial::NotAdmittedUpsert
        | Denial::NotAdmittedTerminalHeadRetirement
        | Denial::SourceRoot
        | Denial::SourcePath
        | Denial::Read => return HistoricalFailure::Invalid,
    };
    match past.dimension() {
        HeadReplayBound::EffectBytes => staging
            .beside(past.observed(), past.admitted())
            .map_or(HistoricalFailure::CountOverflow, |limit| {
                HistoricalFailure::Limit(PageLimit::Recovery(limit))
            }),
        HeadReplayBound::HeapBytes => {
            let _ = resident.transient(past.observed());
            HistoricalFailure::Invalid
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_store_recovery_physics::{test_support::head_replay_limit_for_test, HeadReplayBound};

    use super::*;
    use crate::orchestration::recovery_budget::{allowance_for_test, recovery_limit_for_test};

    const STAGING: RecoveryAllowance =
        allowance_for_test(PhysicalRecoveryLimitDimension::StagingBytes, 896);

    #[test]
    fn a_bound_physics_refused_is_the_limit_that_ran_out_and_not_damage() {
        let past = |bound, observed, admitted| {
            SelectedReleaseHeadReplayDenial::BoundExceeded(head_replay_limit_for_test(
                bound, observed, admitted,
            ))
        };
        let mut resident = ResidentAllowance::new(64);
        resident.bytes(24).unwrap();
        // Physics was admitted all 896 staging bytes and needed 897.
        assert_eq!(
            refused(
                past(HeadReplayBound::EffectBytes, 897, 896),
                STAGING,
                &mut resident
            ),
            HistoricalFailure::Limit(PageLimit::Recovery(recovery_limit_for_test(
                PhysicalRecoveryLimitDimension::StagingBytes,
                897,
                896
            )))
        );
        assert_eq!(
            refused(
                SelectedReleaseHeadReplayDenial::SourcePath,
                STAGING,
                &mut resident
            ),
            HistoricalFailure::Invalid
        );
        assert_eq!(
            refused(
                SelectedReleaseHeadReplayDenial::SizeOverflow,
                STAGING,
                &mut resident
            ),
            HistoricalFailure::CountOverflow
        );
        assert_eq!(resident.exceeded_requirement(), None);
        // Forty bytes were left and the replay needed 41 at once.
        assert_eq!(
            refused(
                past(HeadReplayBound::HeapBytes, 41, 40),
                STAGING,
                &mut resident
            ),
            HistoricalFailure::Invalid
        );
        assert_eq!(resident.exceeded_requirement(), Some(65));
    }
}
