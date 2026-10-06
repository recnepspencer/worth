use worth_store_physical_format::RecordArtifactFile;

use super::artifact_generation;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::integrity_ingress::projection::MembershipProjectionFailure;
use crate::orchestration::planning::manifest_entry_budget::{
    ChargeTarget, ChargeToken, EntriesStopped, EntryAdmission, ManifestEntryBudget, ROOT_ENTRY,
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

/// Charges the candidate root of `generation` its one entry, before the
/// root is probed. An absent root still costs that entry: the probe looked.
/// The same token then pays for every tree block read under a present root.
pub(super) fn charge_successor_root(
    budget: &mut ManifestEntryBudget,
    generation: u64,
) -> Result<ChargeToken, PhysicalRecoverySuccessorCandidateDenial> {
    let artifact = RecordArtifactFile::RootManifest { generation };
    budget
        .charge(ROOT_ENTRY, ChargeTarget::root(generation))
        .map_err(|refused| stopped(artifact, refused))
}

pub(super) fn consume_successor(
    budget: &mut ManifestEntryBudget,
    entries: usize,
    artifact: RecordArtifactFile,
) -> Result<(), PhysicalRecoverySuccessorCandidateDenial> {
    budget
        .admit(entries)
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
    use crate::orchestration::planning::manifest_entry_budget::{
        manifest_entry_limit_for_test, EntriesStopped,
    };

    #[test]
    fn the_candidate_root_is_charged_one_entry_and_refused_as_that_limit_with_its_value() {
        let artifact = RecordArtifactFile::RootManifest { generation: 9 };
        let mut none_left = ManifestEntryBudget::for_test(8, 8);
        assert_eq!(
            charge_successor_root(&mut none_left, 9).err(),
            Some(
                PhysicalRecoverySuccessorCandidateDenial::ManifestEntryLimit {
                    artifact,
                    generation: 9,
                }
            )
        );
        assert_eq!(
            none_left.refused().map(EntriesStopped::Limit),
            Some(EntriesStopped::Limit(manifest_entry_limit_for_test(9, 8)))
        );
        // One left pays for the root, and for every block read under it.
        let mut one_left = ManifestEntryBudget::for_test(8, 7);
        assert!(charge_successor_root(&mut one_left, 9).is_ok());
        assert_eq!(one_left.remaining(), 0);
    }
}
