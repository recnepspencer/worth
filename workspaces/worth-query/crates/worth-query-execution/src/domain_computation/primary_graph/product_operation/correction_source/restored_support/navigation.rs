//! A history document's navigation commits in a correction's restored
//! support (tree-based undo S3, ruling B, refined).
//!
//! Undo, redo and checkout restore a recorded state through one owned-graph
//! selection commit. That commit writes every selected entity, and it is not
//! an application commit, so it has no correction witness. Without this rule,
//! any correction after a navigation refused `CanonicalEvidenceUnavailable`.
//!
//! The rule, and why it is sound:
//! - A commit is a navigation only if its canonical commit envelope is an
//!   owned-graph selection. Its kind is read, never inferred from a missing
//!   witness.
//! - A navigation restores the full recorded state of the owned closure, and
//!   the original bases retain their payloads. So the latest navigation
//!   commit before the target that touches the facts decides what those facts
//!   hold, and every occurrence up to it (earlier navigations, and writes on
//!   a branch it navigated away from) is superseded by the state it restored.
//!   Undo, then redo, of the source step is the everyday case.
//! - Those occurrences count as cancelled only if the history document says
//!   the basis the latest navigation selected is a recorded step on the
//!   selected line that descends from the correction's source step. That is
//!   an O(log) index and level-ancestor query, never a walk. Otherwise every
//!   occurrence still needs its correction witness.
//! - Every occurrence after the latest navigation still needs its correction
//!   witness, as without this rule.
//! - The final comparison of the original facts with the target
//!   (`eligibility::compare`) still runs on what is actually present, and it
//!   is what refuses a navigation that restored a state where the source's
//!   occurrence is no longer the one present (`SourceDiverged`).
use super::super::{WorthQueryCorrectionDerivationDenial as Denial, WorthQuerySelectedCorrectionSource};
use super::cancellation::Occurrences;
use super::WorthQueryCorrectionRestoredSupportDenial as Stop;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational::facade::{history::CommitId, replay::CanonicalCommitAuthorityKind, runtime::RelationalRuntime};

/// What a history document answers about its own navigation commits.
pub trait WorthQueryCorrectionNavigationLineage {
    /// Whether the recorded step whose retained basis is the relational
    /// commit `selected` is on the document's selected line and descends from
    /// the recorded step whose stages made `source`.
    fn restores_descendant_of(
        &self,
        selected: CommitId,
        source: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) -> bool;
}

/// An occurrence as the rule sees it: a navigation and the basis it
/// selected, or anything else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OccurrenceKind {
    Navigation { selected: Option<CommitId> },
    Other,
}

/// The latest navigation commit (the highest), when the basis it selected is
/// one `vouch` accepts: every occurrence up to it is superseded.
pub(super) fn superseded_through(
    kinds: impl Iterator<Item = (CommitId, OccurrenceKind)>,
    vouch: impl Fn(CommitId) -> bool,
) -> Option<CommitId> {
    kinds
        .filter_map(|(commit, kind)| match kind {
            OccurrenceKind::Navigation { selected } => Some((commit, selected)),
            OccurrenceKind::Other => None,
        })
        .max_by_key(|(commit, _)| *commit)
        .filter(|(_, selected)| selected.is_some_and(&vouch))
        .map(|(commit, _)| commit)
}

/// Whether the occurrence at `commit` still needs its correction witness.
pub(super) fn needs_witness(commit: CommitId, superseded: Option<CommitId>) -> bool {
    superseded.is_none_or(|through| commit > through)
}

/// `occurrences` without the navigation commits the rule cancels.
pub(super) fn without_vouched_navigation<Schema>(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    occurrences: Occurrences,
    lineage: Option<&dyn WorthQueryCorrectionNavigationLineage>,
) -> Result<Occurrences, Denial> {
    let Some(lineage) = lineage else {
        return Ok(occurrences);
    };
    let mut kinds = ExecutionOrderedMap::<CommitId, OccurrenceKind>::default();
    for (commit, _) in &occurrences {
        super::ordered_visit(execution, occurrences.len()).map_err(Denial::RestoredSupport)?;
        let envelope = runtime
            .replay()
            .canonical_commit_envelope(&execution, *commit)
            .map_err(|cause| Denial::RestoredSupport(Stop::Execution(cause)))?;
        let kind = match envelope.map(|envelope| envelope.authority_kind()) {
            Some(CanonicalCommitAuthorityKind::OwnedGraphSelection(selection)) => OccurrenceKind::Navigation {
                selected: match selection.selected_basis().reference().target() {
                    worth_foundational::FoundationalBranchTarget::Basis(target) => {
                        Some(CommitId(target.selected_commit_id()))
                    }
                    worth_foundational::FoundationalBranchTarget::Empty => None,
                },
            },
            _ => OccurrenceKind::Other,
        };
        super::ordered_visit(execution, kinds.len()).map_err(Denial::RestoredSupport)?;
        kinds
            .insert_cloned_native_for_request(commit, &kind, execution)
            .map_err(|cause| Denial::RestoredSupport(Stop::Storage(cause)))?;
    }
    let superseded = superseded_through(kinds.iter().map(|(commit, kind)| (*commit, *kind)), |selected| {
        lineage.restores_descendant_of(selected, source.source_commit())
    });
    let mut kept = Occurrences::default();
    for (commit, occurrence) in &occurrences {
        if needs_witness(*commit, superseded) {
            super::ordered_visit(execution, kept.len()).map_err(Denial::RestoredSupport)?;
            kept.insert_cloned_native_for_request(commit, occurrence, execution)
                .map_err(|cause| Denial::RestoredSupport(Stop::Storage(cause)))?;
        }
    }
    Ok(kept)
}

#[cfg(test)]
mod tests {
    use super::{needs_witness, superseded_through, OccurrenceKind::*};
    use worth_relational::facade::history::CommitId;

    fn navigation(basis: u64) -> super::OccurrenceKind {
        Navigation { selected: Some(CommitId(basis)) }
    }

    fn vouch(accepted: u64) -> impl Fn(CommitId) -> bool {
        move |basis| basis == CommitId(accepted)
    }

    /// M1: only a navigation commit's kind supersedes. A write after the
    /// latest navigation still needs its witness; counting it as a navigation
    /// (with no basis) would make it the latest, unvouched, and supersede
    /// nothing.
    #[test]
    fn only_a_navigation_supersedes_and_a_later_write_needs_its_witness() {
        let kinds = [(CommitId(10), navigation(2)), (CommitId(11), Other)];
        let superseded = superseded_through(kinds.into_iter(), vouch(2));
        assert_eq!(superseded, Some(CommitId(10)));
        assert!(!needs_witness(CommitId(10), superseded));
        assert!(needs_witness(CommitId(11), superseded));
        let as_navigation = [(CommitId(10), navigation(2)), (CommitId(11), Navigation { selected: None })];
        assert_eq!(superseded_through(as_navigation.into_iter(), vouch(2)), None);
    }

    /// Only the latest navigation decides: undo (basis before the source),
    /// then redo (basis at the source) supersedes both; the reverse order
    /// supersedes nothing.
    #[test]
    fn the_latest_navigation_decides_not_the_first() {
        let undo_then_redo = [(CommitId(10), navigation(1)), (CommitId(12), navigation(2))];
        assert_eq!(superseded_through(undo_then_redo.into_iter(), vouch(2)), Some(CommitId(12)));
        assert_eq!(superseded_through(undo_then_redo.into_iter(), vouch(1)), None);
        let away_write_back = [(CommitId(10), navigation(1)), (CommitId(11), Other), (CommitId(12), navigation(2))];
        let superseded = superseded_through(away_write_back.into_iter(), vouch(2));
        assert!(!needs_witness(CommitId(11), superseded), "a write before the latest navigation is superseded");
        assert_eq!(superseded_through([(CommitId(9), Other)].into_iter(), |_| true), None);
        assert_eq!(superseded_through([(CommitId(10), Navigation { selected: None })].into_iter(), |_| true), None);
        assert!(needs_witness(CommitId(1), None));
    }
}
