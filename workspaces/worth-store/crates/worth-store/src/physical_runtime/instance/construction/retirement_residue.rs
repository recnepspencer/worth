use super::super::durability_bootstrap::ReopenedPhysicalDurabilityOwners;
use crate::physical_runtime::record_serving::RecordServingState;

/// Candidate residue is admitted only by an exact retained retirement or
/// unpublished copy obligation. Candidate bytes remain untrusted.
pub(super) enum PublicationResidueAdmission {
    Clean,
    RecoverableRetirementCandidate,
    RecoverableCopyDestination,
    InspectionRequired,
}

impl PublicationResidueAdmission {
    pub(super) fn classify(
        state: &RecordServingState,
        durability: &ReopenedPhysicalDurabilityOwners,
    ) -> Self {
        if state.publication_residue.is_empty() {
            return Self::Clean;
        }
        let sealed = durability.wal.observation().sealed_for_inspection();
        if state.publication_residue.only_next_arena_artifact() {
            let obligations = durability.wal.recovered_copy_obligations();
            let [record] = obligations.as_slice() else {
                return Self::InspectionRequired;
            };
            let intent = record.intent();
            let claim = CopyResidueClaim {
                source_root: intent.source_root(),
                arena: intent.destination().arena_range().arena().get(),
                offset: intent.destination().arena_range().offset(),
                published: record.publication().is_some(),
                resolved: record.resolution().is_some(),
            };
            return if admissible_copy_residue(
                sealed,
                durability.unresolved_retirements.len(),
                state.current_root.generation(),
                state.free_space.next_arena(),
                &[claim],
            ) {
                // Director seeding still proves the source graph, retains its
                // lease and installs the exact private destination hold.
                Self::RecoverableCopyDestination
            } else {
                Self::InspectionRequired
            };
        }
        if !state
            .publication_residue
            .permits_retirement_candidate_reconstruction()
            || sealed
        {
            return Self::InspectionRequired;
        }
        let [record] = durability.unresolved_retirements.as_slice() else {
            return Self::InspectionRequired;
        };
        let Some(release) = record.release else {
            return Self::InspectionRequired;
        };
        if record.completion
            || !matches!(
                record.artifact,
                crate::physical_runtime::durability::RetiredArtifact::Extent { .. }
                    | crate::physical_runtime::durability::RetiredArtifact::Arena { .. }
            )
            || release.source_generation() != state.current_root.generation()
            || state.current_root.generation().checked_add(1)
                != Some(release.candidate_generation())
        {
            return Self::InspectionRequired;
        }
        // Director construction installs the exclusive pending lease from this
        // same obligation before the assembled serving runtime is exposed.
        Self::RecoverableRetirementCandidate
    }
}

#[derive(Clone, Copy)]
struct CopyResidueClaim {
    source_root: u64,
    arena: u64,
    offset: u64,
    published: bool,
    resolved: bool,
}

fn admissible_copy_residue(
    sealed: bool,
    retirements: usize,
    current_root: u64,
    next_arena: u64,
    claims: &[CopyResidueClaim],
) -> bool {
    let [claim] = claims else {
        return false;
    };
    !sealed
        && retirements == 0
        && claim.source_root > 0
        && claim.source_root <= current_root
        && claim.arena == next_arena
        && claim.offset == 0
        && !claim.published
        && !claim.resolved
}

#[cfg(test)]
mod tests {
    use super::{admissible_copy_residue, CopyResidueClaim};

    #[test]
    fn only_one_unresolved_unpublished_matching_wal_copy_explains_next_arena() {
        let matching = CopyResidueClaim {
            source_root: 3,
            arena: 8,
            offset: 0,
            published: false,
            resolved: false,
        };
        assert!(admissible_copy_residue(false, 0, 4, 8, &[matching]));
        assert!(
            !admissible_copy_residue(true, 0, 4, 8, &[matching]),
            "sealed WAL"
        );
        assert!(
            !admissible_copy_residue(false, 1, 4, 8, &[matching]),
            "competing retirement"
        );
        assert!(
            !admissible_copy_residue(false, 0, 4, 8, &[]),
            "no copy authority"
        );
        assert!(
            !admissible_copy_residue(false, 0, 4, 8, &[matching, matching]),
            "multiple copies"
        );
        assert!(
            !admissible_copy_residue(false, 0, 4, 9, &[matching]),
            "wrong arena"
        );
        assert!(
            !admissible_copy_residue(false, 0, 2, 8, &[matching]),
            "future source root"
        );
        assert!(!admissible_copy_residue(
            false,
            0,
            4,
            8,
            &[CopyResidueClaim {
                offset: 4096,
                ..matching
            }]
        ));
        assert!(!admissible_copy_residue(
            false,
            0,
            4,
            8,
            &[CopyResidueClaim {
                published: true,
                ..matching
            }]
        ));
        assert!(!admissible_copy_residue(
            false,
            0,
            4,
            8,
            &[CopyResidueClaim {
                resolved: true,
                ..matching
            }]
        ));
    }
}
