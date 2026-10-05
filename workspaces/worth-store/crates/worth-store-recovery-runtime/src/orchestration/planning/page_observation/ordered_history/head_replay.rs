//! Bind one historical released edge's WAL-chosen head path to its addressed
//! source-tree blocks while the bounded discovery cursor is still live.

use worth_store::physical_runtime::{BoundedRecoveryFilesystemDiscovery, RecoveryDiscoveryFailure};
use worth_store_physical_format::{DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration};
use worth_store_recovery_physics::{
    AdmittedRootStepMemberView, SelectedReleaseHeadReplayDenial,
    VerifiedOrderedReleasedHeadReplayV14, VerifiedReleasedRootEdge,
    VerifiedSelectedReleaseHeadReplayV14,
};

use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;

use super::walk_failure::WalkFailure;

/// Physics carries no reason across its reader, so the reader keeps it.
#[derive(Default)]
struct Unread(Option<WalkFailure>);

impl Unread {
    fn keep(&mut self, failure: RecoveryDiscoveryFailure) {
        self.0 = Some(failure.into());
    }

    /// A failed read's own verdict; without one, what physics refused.
    fn verdict(self, denial: SelectedReleaseHeadReplayDenial, maximum_scratch: u64) -> WalkFailure {
        self.0
            .unwrap_or_else(|| WalkFailure::replay_refused(denial, maximum_scratch))
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
    maximum_scratch: u64,
) -> Result<VerifiedSelectedReleaseHeadReplayV14, WalkFailure> {
    // Replaying one member's head path is one lookup, however many blocks
    // the path crosses.
    budget.consume(1)?;
    let mut unread = Unread::default();
    VerifiedSelectedReleaseHeadReplayV14::admit_addressed_member(
        member,
        source,
        result,
        format,
        maximum_effect_bytes,
        maximum_effect_bytes,
        |reference, maximum| {
            discovery
                .read_release_custody_head_block(reference.generation(), reference.block(), maximum)
                .map_err(|failure| unread.keep(failure))?
                .bytes()
                .map(<[u8]>::to_vec)
                .ok_or(())
        },
    )
    .map_err(|denial| unread.verdict(denial, maximum_scratch))
}

pub(super) fn bind_edge(
    edge: &VerifiedReleasedRootEdge,
    replay: VerifiedSelectedReleaseHeadReplayV14,
    source: &DurablePhysicalRootManifest,
    result: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    remaining_additional_heap_bytes: u64,
    maximum_scratch: u64,
) -> Result<VerifiedOrderedReleasedHeadReplayV14, WalkFailure> {
    VerifiedOrderedReleasedHeadReplayV14::bind_edge(
        edge,
        replay,
        source,
        result,
        format,
        remaining_additional_heap_bytes,
    )
    .map_err(|denial| WalkFailure::replay_refused(denial, maximum_scratch))
}

#[cfg(test)]
mod tests {
    use worth_store::physical_runtime::{FilesystemObservationBound, RecoveryDiscoveryArtifact};

    use super::*;
    use crate::orchestration::reader_limit::refused_past;
    use SelectedReleaseHeadReplayDenial as Denial;

    #[test]
    fn a_replay_a_read_stopped_keeps_the_reads_verdict() {
        let mut out_of_bytes = Unread::default();
        out_of_bytes.keep(refused_past(
            FilesystemObservationBound::ObservationBytes,
            65_537,
            65_536,
        ));
        assert_eq!(
            out_of_bytes.verdict(Denial::Read, 100),
            WalkFailure::ByteLimit
        );
        let mut damaged = Unread::default();
        damaged.keep(RecoveryDiscoveryFailure::InvalidAddress {
            artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
        });
        assert_eq!(damaged.verdict(Denial::Read, 100), WalkFailure::Unverified);
    }

    #[test]
    fn a_replay_physics_refused_is_what_physics_says() {
        use worth_store_recovery_physics::{
            test_support::head_replay_limit_for_test, HeadReplayBound,
        };
        let refused = |denial| Unread::default().verdict(denial, 100);
        let past = head_replay_limit_for_test(HeadReplayBound::EffectBytes, 70, 60);
        assert_eq!(
            refused(Denial::BoundExceeded(past)),
            WalkFailure::ScratchLimit { at_least: 110 }
        );
        assert_eq!(refused(Denial::SourcePath), WalkFailure::Unverified);
        assert_eq!(refused(Denial::Read), WalkFailure::Unverified);
    }
}
