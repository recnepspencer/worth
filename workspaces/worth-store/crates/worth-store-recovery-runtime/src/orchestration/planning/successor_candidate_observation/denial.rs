use worth_store_physical_format::RecordArtifactFile;

use super::artifact_generation;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::integrity_ingress::projection::MembershipProjectionFailure;
use crate::orchestration::planning::manifest_entry_budget::{ManifestEntryBudget, RootUnit};

pub(super) const fn invalid(
    artifact: RecordArtifactFile,
) -> PhysicalRecoverySuccessorCandidateDenial {
    PhysicalRecoverySuccessorCandidateDenial::InvalidArtifact {
        artifact,
        generation: artifact_generation(artifact),
    }
}

const fn limit(
    artifact: RecordArtifactFile,
    observed: u64,
    admitted: u64,
) -> PhysicalRecoverySuccessorCandidateDenial {
    PhysicalRecoverySuccessorCandidateDenial::ManifestEntryLimit {
        artifact,
        generation: artifact_generation(artifact),
        observed,
        admitted,
    }
}

/// Charges the candidate root its one entry, before its leaves are read.
/// Reading a block charges nothing, so nothing else is refused before them.
pub(super) fn charge_successor_root(
    budget: &mut ManifestEntryBudget,
    artifact: RecordArtifactFile,
) -> Result<RootUnit, PhysicalRecoverySuccessorCandidateDenial> {
    budget
        .charge_root_with_evidence()
        .map_err(|(observed, admitted)| limit(artifact, observed, admitted))
}

pub(super) fn consume_successor(
    budget: &mut ManifestEntryBudget,
    entries: usize,
    artifact: RecordArtifactFile,
) -> Result<(), PhysicalRecoverySuccessorCandidateDenial> {
    budget
        .consume_with_evidence(entries)
        .map_err(|(observed, admitted)| limit(artifact, observed, admitted))
}

pub(super) fn membership_failure(
    budget: &ManifestEntryBudget,
    artifact: RecordArtifactFile,
    failure: MembershipProjectionFailure,
) -> PhysicalRecoverySuccessorCandidateDenial {
    match failure {
        MembershipProjectionFailure::EntryLimit { observed } => {
            let (observed, admitted) = budget.crossing_evidence(observed);
            limit(artifact, observed, admitted)
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

    #[test]
    fn the_candidate_root_is_charged_one_entry_and_refused_as_that_limit_with_its_value() {
        let artifact = RecordArtifactFile::RootManifest { generation: 9 };
        let mut none_left = ManifestEntryBudget::new(8, 8);
        assert_eq!(
            charge_successor_root(&mut none_left, artifact).err(),
            Some(limit(artifact, 9, 8))
        );
        // One left pays for the root, and for every block read under it.
        let mut one_left = ManifestEntryBudget::new(8, 7);
        assert!(charge_successor_root(&mut one_left, artifact).is_ok());
        assert_eq!(one_left.remaining(), 0);
    }
}
