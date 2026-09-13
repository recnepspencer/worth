use super::counters::OwnerSemanticVerificationCounters;
use super::row_verification::VerifiedOwnerRow;
use crate::backup_verification::BackupVerificationDefect;
use crate::inspection::OwnerDecodedArtifactBinding;
use crate::truth_composition::RecoveryCandidateObservation;
use worth_store_physical_format::RootPublicationCell;

pub(super) fn record(
    mut counters: OwnerSemanticVerificationCounters,
    row: VerifiedOwnerRow,
    defects: &mut Vec<BackupVerificationDefect>,
    recovery_candidates: &mut Vec<RecoveryCandidateObservation>,
    owner_bindings: &mut Vec<OwnerDecodedArtifactBinding>,
    expected_root: &mut Option<RootPublicationCell>,
) -> OwnerSemanticVerificationCounters {
    if row.root_publication.is_some() {
        *expected_root = row.root_publication;
    }
    let Some(recorded) = counters.record(row.observation) else {
        defects.push(BackupVerificationDefect::VerificationCounterOverflow);
        return counters;
    };
    counters = recorded;
    if let Some(candidate) = row.recovery_candidate {
        recovery_candidates.push(candidate);
    }
    owner_bindings.push(row.owner_binding);
    counters
}
