use worth_store_physical_format::RecordArtifactFile;

use super::artifact_generation;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::integrity_ingress::projection::MembershipProjectionFailure;
use crate::orchestration::planning::manifest_entry_budget::{
    EntriesStopped, ManifestEntryBudget, RootUnit,
};

pub(super) const fn invalid(
    artifact: RecordArtifactFile,
) -> PhysicalRecoverySuccessorCandidateDenial {
    PhysicalRecoverySuccessorCandidateDenial::InvalidArtifact {
        artifact,
        generation: artifact_generation(artifact),
    }
}

/// The budget's refusal, for the candidate it stopped. A count past every
/// count is no limit: no valid artifact holds that many entries.
fn stopped(
    artifact: RecordArtifactFile,
    stopped: EntriesStopped,
) -> PhysicalRecoverySuccessorCandidateDenial {
    match stopped {
        // The budget holds the refusal's counts for the block's cause.
        EntriesStopped::Limit(_) => PhysicalRecoverySuccessorCandidateDenial::ManifestEntryLimit {
            artifact,
            generation: artifact_generation(artifact),
        },
        EntriesStopped::CountOverflow => invalid(artifact),
    }
}

/// Charges the candidate root its one entry, before its leaves are read.
/// Reading a block charges nothing, so nothing else is refused before them.
pub(super) fn charge_successor_root(
    budget: &mut ManifestEntryBudget,
    artifact: RecordArtifactFile,
) -> Result<RootUnit, PhysicalRecoverySuccessorCandidateDenial> {
    budget
        .charge_root()
        .map_err(|refused| stopped(artifact, refused))
}

pub(super) fn consume_successor(
    budget: &mut ManifestEntryBudget,
    entries: usize,
    artifact: RecordArtifactFile,
) -> Result<(), PhysicalRecoverySuccessorCandidateDenial> {
    budget
        .charge(entries)
        .map_err(|refused| stopped(artifact, refused))
}

pub(super) fn membership_failure(
    budget: &mut ManifestEntryBudget,
    artifact: RecordArtifactFile,
    failure: MembershipProjectionFailure,
) -> PhysicalRecoverySuccessorCandidateDenial {
    match failure {
        MembershipProjectionFailure::EntryLimit { observed } => {
            stopped(artifact, budget.refuse_decoded(observed))
        }
        MembershipProjectionFailure::Integrity(rejection) => {
            PhysicalRecoverySuccessorCandidateDenial::RootProtocol {
                artifact,
                generation: artifact_generation(artifact),
                denial: rejection.diagnostic(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::PhysicalRecoveryLimitDimension::ManifestEntries;
    use crate::orchestration::planning::manifest_entry_budget::EntriesStopped;
    use crate::orchestration::recovery_budget::recovery_limit_for_test;

    #[test]
    fn the_candidate_root_is_charged_one_entry_and_refused_as_that_limit_with_its_value() {
        let artifact = RecordArtifactFile::RootManifest { generation: 9 };
        let mut none_left = ManifestEntryBudget::new(8, 8);
        assert_eq!(
            charge_successor_root(&mut none_left, artifact).err(),
            Some(
                PhysicalRecoverySuccessorCandidateDenial::ManifestEntryLimit {
                    artifact,
                    generation: 9,
                }
            )
        );
        assert_eq!(
            none_left.refused().map(EntriesStopped::Limit),
            Some(EntriesStopped::Limit(recovery_limit_for_test(
                ManifestEntries,
                9,
                8
            )))
        );
        // One left pays for the root, and for every block read under it.
        let mut one_left = ManifestEntryBudget::new(8, 7);
        assert!(charge_successor_root(&mut one_left, artifact).is_ok());
        assert_eq!(one_left.remaining(), 0);
    }
}
