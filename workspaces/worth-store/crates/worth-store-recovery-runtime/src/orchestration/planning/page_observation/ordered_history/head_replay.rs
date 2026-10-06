//! Bind one historical released edge's WAL-chosen head path to its addressed
//! source-tree blocks while the bounded discovery cursor is still live.

use crate::orchestration::recovery_budget::RecoveryAllowance;
use worth_store::physical_runtime::{
    ArtifactCeiling, BoundedRecoveryFilesystemDiscovery, PageAddress, ReadGrant,
    RecoveryDiscoveryFailure, UnchargedRead,
};
use worth_store_physical_format::{DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration};
use worth_store_recovery_physics::{
    AdmittedRootStepMemberView, SelectedReleaseHeadReplayDenial,
    VerifiedOrderedReleasedHeadReplayV14, VerifiedReleasedRootEdge,
    VerifiedSelectedReleaseHeadReplayV14,
};

use crate::orchestration::planning::manifest_entry_budget::{EntryAdmission, ManifestEntryBudget};

use super::walk_failure::WalkFailure;

/// Physics carries no reason across its reader, so the reader keeps it.
#[derive(Default)]
struct Unread(Option<WalkFailure>);

impl Unread {
    fn keep(&mut self, failure: RecoveryDiscoveryFailure) {
        self.0 = Some(failure.into());
    }

    /// A failed read's own verdict; without one, what physics refused.
    fn verdict(
        self,
        denial: SelectedReleaseHeadReplayDenial,
        staging: RecoveryAllowance,
    ) -> WalkFailure {
        self.0
            .unwrap_or_else(|| WalkFailure::replay_refused(denial, staging))
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn admit_addressed_member(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    budget: &mut ManifestEntryBudget,
    member: AdmittedRootStepMemberView<'_>,
    source: &DurablePhysicalRootManifest,
    result: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    maximum_effect_bytes: u64,
    staging: RecoveryAllowance,
) -> Result<VerifiedSelectedReleaseHeadReplayV14, WalkFailure> {
    // Replaying one member's head path is one lookup, however many blocks
    // the path crosses.
    budget.admit(1)?;
    let mut unread = Unread::default();
    VerifiedSelectedReleaseHeadReplayV14::admit_addressed_member(
        member,
        source,
        result,
        format,
        maximum_effect_bytes,
        maximum_effect_bytes,
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
                .map_err(|failure| unread.keep(failure))?
                .bytes()
                .map(<[u8]>::to_vec)
                .ok_or(())
        },
    )
    .map_err(|denial| unread.verdict(denial, staging))
}

pub(super) fn bind_edge(
    edge: &VerifiedReleasedRootEdge,
    replay: VerifiedSelectedReleaseHeadReplayV14,
    source: &DurablePhysicalRootManifest,
    result: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    remaining_additional_heap_bytes: u64,
    staging: RecoveryAllowance,
) -> Result<VerifiedOrderedReleasedHeadReplayV14, WalkFailure> {
    VerifiedOrderedReleasedHeadReplayV14::bind_edge(
        edge,
        replay,
        source,
        result,
        format,
        remaining_additional_heap_bytes,
    )
    .map_err(|denial| WalkFailure::replay_refused(denial, staging))
}

#[cfg(test)]
mod tests {
    use worth_store::physical_runtime::{
        ArtifactDamage, FilesystemObservationBound, RecoveryDiscoveryArtifact,
    };

    use super::*;
    use crate::entry::PhysicalRecoveryLimitDimension::{ObservationBytes, StagingBytes};
    use crate::orchestration::planning::page_observation::PageLimit;
    use crate::orchestration::reader_limit::refused_past;
    use crate::orchestration::recovery_budget::{allowance_for_test, recovery_limit_for_test};
    use SelectedReleaseHeadReplayDenial as Denial;

    const STAGING: RecoveryAllowance = allowance_for_test(StagingBytes, 100);

    #[test]
    fn a_replay_a_read_stopped_keeps_the_reads_verdict() {
        let mut out_of_bytes = Unread::default();
        out_of_bytes.keep(refused_past(
            FilesystemObservationBound::ObservationBytes,
            65_537,
            65_536,
        ));
        // The reader was handed 65,536 of the 65,540 observation bytes.
        let WalkFailure::Limit(limit) = out_of_bytes.verdict(Denial::Read, STAGING) else {
            panic!("a reader out of bytes is a limit");
        };
        assert_eq!(
            limit.in_recovery(
                &crate::entry::PhysicalRecoveryLimitDeclaration::observing_for_test(65_540)
            ),
            Some(recovery_limit_for_test(ObservationBytes, 65_541, 65_540).into()),
        );
        let mut damaged = Unread::default();
        damaged.keep(RecoveryDiscoveryFailure::Damage(
            ArtifactDamage::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
            },
        ));
        assert_eq!(
            damaged.verdict(Denial::Read, STAGING),
            WalkFailure::Unverified
        );
    }

    #[test]
    fn a_replay_physics_refused_is_what_physics_says() {
        use worth_store_recovery_physics::{
            test_support::head_replay_limit_for_test, HeadReplayBound,
        };
        let refused = |denial| Unread::default().verdict(denial, STAGING);
        let past = head_replay_limit_for_test(HeadReplayBound::EffectBytes, 70, 60);
        assert_eq!(
            refused(Denial::BoundExceeded(past)),
            WalkFailure::Limit(PageLimit::Recovery(recovery_limit_for_test(
                StagingBytes,
                110,
                100
            )))
        );
        assert_eq!(refused(Denial::SourcePath), WalkFailure::Unverified);
        assert_eq!(refused(Denial::Read), WalkFailure::Unverified);
    }
}
