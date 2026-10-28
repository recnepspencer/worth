//! Adjudication of a killed runtime's Store checkpoint obligations against
//! the checkpoint this recovery selected.
//!
//! A Store checkpoint builds candidate `selected + 1` in staging (create,
//! append, synchronize), replaces `checkpoint.current` with it, and then
//! synchronizes that namespace. Recovery selects at most one checkpoint
//! through its own admission (none is sequence 0 here), so each obligation
//! names either the candidate recovery did not select or the checkpoint it
//! did:
//!
//! - `selected + 1`, any action: recovery found `checkpoint.current` still
//!   holding its selection (or still absent), and its residue stage removed
//!   the candidate or observed it absent. Nothing of that attempt is
//!   reachable.
//! - `selected`, any action but removal: the replacement reached
//!   `checkpoint.current` and recovery admitted those exact bytes. A
//!   replacement starts only after the candidate is synchronized, so recovery
//!   settles it by synchronizing the publication namespace itself.
//! - Everything else is retained: a removal of the candidate that later
//!   became selected contradicts the publication order, and recovery
//!   observed no candidate other than `selected + 1`.

use worth_store::physical_runtime::PhysicalCheckpointRecoveryAction;

use crate::progression::ReopenedPhysicalRecovery;

/// The checkpoint recovery selected (0 when it observed none), once its
/// residue stage has settled the next candidate (removed or proven absent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SettledCheckpointResidue {
    selected: u64,
}

impl SettledCheckpointResidue {
    /// Only cleanup calls this, after the residue stage answered `Absent` or
    /// `Removed`; a refused or indeterminate residue stage ends recovery first,
    /// and that stage refuses a recovery that never observed the checkpoint
    /// slot, so no selection here is an observed absence.
    pub(super) fn after_residue_stage(reopened: &ReopenedPhysicalRecovery) -> Self {
        Self {
            selected: reopened
                .state
                .coordination
                .owner()
                .checkpoint()
                .map_or(0, |checkpoint| {
                    checkpoint.facts().source().identity().sequence().get()
                }),
        }
    }
}

/// How one checkpoint obligation is settled, if it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CheckpointObligationVerdict {
    /// The candidate was not selected and its residue is gone.
    UnselectedCandidateRemoved,
    /// Recovery selected this checkpoint; it is settled once recovery has
    /// made the publication namespace durable.
    SelectedPublication,
    Retained,
}

pub(super) fn adjudicate(
    residue: SettledCheckpointResidue,
    sequence: u64,
    action: PhysicalCheckpointRecoveryAction,
) -> CheckpointObligationVerdict {
    let selected = residue.selected;
    if selected.checked_add(1) == Some(sequence) {
        return CheckpointObligationVerdict::UnselectedCandidateRemoved;
    }
    if sequence != selected || selected == 0 {
        return CheckpointObligationVerdict::Retained;
    }
    match action {
        PhysicalCheckpointRecoveryAction::CreateCandidate { .. }
        | PhysicalCheckpointRecoveryAction::AppendCandidate { .. }
        | PhysicalCheckpointRecoveryAction::SynchronizeCandidate
        | PhysicalCheckpointRecoveryAction::PublishCandidate
        | PhysicalCheckpointRecoveryAction::SynchronizeNamespace => {
            CheckpointObligationVerdict::SelectedPublication
        }
        PhysicalCheckpointRecoveryAction::RemoveCandidate => CheckpointObligationVerdict::Retained,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use CheckpointObligationVerdict::{Retained, SelectedPublication, UnselectedCandidateRemoved};
    use PhysicalCheckpointRecoveryAction as Action;

    const ACTIONS: [Action; 6] = [
        Action::CreateCandidate { byte_count: 1 },
        Action::AppendCandidate {
            offset: 1,
            byte_count: 1,
        },
        Action::SynchronizeCandidate,
        Action::RemoveCandidate,
        Action::PublishCandidate,
        Action::SynchronizeNamespace,
    ];

    fn selected(sequence: u64) -> SettledCheckpointResidue {
        SettledCheckpointResidue { selected: sequence }
    }

    #[test]
    fn the_unselected_next_candidate_is_settled_by_its_removed_residue() {
        for action in ACTIONS {
            assert_eq!(
                adjudicate(selected(4), 5, action),
                UnselectedCandidateRemoved
            );
            // No checkpoint was selected: the first candidate is the next.
            assert_eq!(
                adjudicate(selected(0), 1, action),
                UnselectedCandidateRemoved
            );
        }
    }

    #[test]
    fn the_selected_checkpoint_is_settled_by_its_durable_publication_except_a_removal() {
        for action in ACTIONS {
            let expected = if action == Action::RemoveCandidate {
                Retained
            } else {
                SelectedPublication
            };
            assert_eq!(adjudicate(selected(5), 5, action), expected, "{action:?}");
        }
    }

    #[test]
    fn candidates_recovery_never_observed_are_retained() {
        for action in ACTIONS {
            assert_eq!(adjudicate(selected(5), 4, action), Retained);
            assert_eq!(adjudicate(selected(5), 7, action), Retained);
            assert_eq!(adjudicate(selected(u64::MAX), 0, action), Retained);
            assert_eq!(adjudicate(selected(0), 0, action), Retained);
            assert_eq!(adjudicate(selected(0), 2, action), Retained);
        }
    }
}
