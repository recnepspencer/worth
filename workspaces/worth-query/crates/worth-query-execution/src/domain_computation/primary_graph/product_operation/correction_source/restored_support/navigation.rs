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
//! - Except a line write: a non-navigation occurrence that the history
//!   document says a recorded step on the path from the source step to the
//!   selected basis wrote. The restored state carries that write, so it is
//!   kept and still needs its witness (public #453). Without this, a write
//!   between the source and a vouched navigation was hidden from the
//!   correction.
//! - Those occurrences count as cancelled only if the history document says
//!   the basis the latest navigation selected is a recorded step on the
//!   selected line that descends from the correction's source step. That is
//!   an O(log) index and level-ancestor query, never a walk. Otherwise every
//!   occurrence still needs its correction witness.
//! - Every occurrence after the latest navigation still needs its correction
//!   witness, as without this rule.
//! - The final comparison of the original facts with the target
//!   (`eligibility::compare`) ALWAYS runs after `without_vouched_navigation`,
//!   on what is actually present. Supersession may cancel writes to facts a
//!   navigation does not restore (outside the owned closure), and the
//!   comparison is then their only guard; skipping it would accept them (a
//!   counterexample the proof trial checked). For owned facts on this path
//!   it is redundant since public #453: a navigation that restored a state
//!   where the source's occurrence is no longer the one present kept the
//!   line writes that changed it, and the cancellation refuses first (court
//!   `a_navigation_that_restored_another_occurrence_still_refuses_the_correction`).
//!   No court kills the mutant that drops it; the Lean
//!   `outside_closure_counterexample` carries its remaining guard.
//! - The descends-from-source check is defense in depth: a latest navigation
//!   to an ancestor of the source precedes the source, so no public path
//!   reaches it, and the unit tests pin it.
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

    /// Whether the relational commit `commit` was written by a recorded step
    /// on the path from the step whose stages made `source` to the step whose
    /// retained basis is `selected`. A document that cannot tell answers true:
    /// the write is then kept and needs its witness.
    fn wrote_on_line_through(
        &self,
        commit: CommitId,
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

/// The latest navigation commit (the highest) and the basis it selected, when
/// `vouch` accepts that basis: occurrences up to it are superseded, except
/// line writes.
pub(super) fn superseded_through(
    kinds: impl Iterator<Item = (CommitId, OccurrenceKind)>,
    vouch: impl Fn(CommitId) -> bool,
) -> Option<(CommitId, CommitId)> {
    kinds
        .filter_map(|(commit, kind)| match kind {
            OccurrenceKind::Navigation { selected } => Some((commit, selected)),
            OccurrenceKind::Other => None,
        })
        .max_by_key(|(commit, _)| *commit)
        .and_then(|(commit, selected)| {
            selected
                .filter(|selected| vouch(*selected))
                .map(|selected| (commit, selected))
        })
}

/// Whether the occurrence at `commit` still needs its correction witness:
/// anything after the superseding navigation, and any line write before it.
pub(super) fn needs_witness(
    commit: CommitId,
    kind: OccurrenceKind,
    superseded: Option<(CommitId, CommitId)>,
    on_line: impl Fn(CommitId, CommitId) -> bool,
) -> bool {
    superseded.is_none_or(|(through, selected)| {
        commit > through || (kind == OccurrenceKind::Other && on_line(commit, selected))
    })
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
    let on_line =
        |commit, selected| lineage.wrote_on_line_through(commit, selected, source.source_commit());
    let mut kept = Occurrences::default();
    for (commit, occurrence) in &occurrences {
        // Every occurrence was classified above; a missing kind keeps the
        // occurrence (it then needs its witness), never drops it.
        let keep = kinds
            .get(commit)
            .is_none_or(|kind| needs_witness(*commit, *kind, superseded, on_line));
        if keep {
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

    fn off_line(_: CommitId, _: CommitId) -> bool {
        false
    }

    /// M1: only a navigation commit's kind supersedes. A write after the
    /// latest navigation still needs its witness; counting it as a navigation
    /// (with no basis) would make it the latest, unvouched, and supersede
    /// nothing.
    #[test]
    fn only_a_navigation_supersedes_and_a_later_write_needs_its_witness() {
        let kinds = [(CommitId(10), navigation(2)), (CommitId(11), Other)];
        let superseded = superseded_through(kinds.into_iter(), vouch(2));
        assert_eq!(superseded, Some((CommitId(10), CommitId(2))));
        assert!(!needs_witness(CommitId(10), navigation(2), superseded, off_line));
        assert!(needs_witness(CommitId(11), Other, superseded, off_line));
        let as_navigation = [(CommitId(10), navigation(2)), (CommitId(11), Navigation { selected: None })];
        assert_eq!(superseded_through(as_navigation.into_iter(), vouch(2)), None);
    }

    /// Only the latest navigation decides: undo (basis before the source),
    /// then redo (basis at the source) supersedes both; the reverse order
    /// supersedes nothing.
    #[test]
    fn the_latest_navigation_decides_not_the_first() {
        let undo_then_redo = [(CommitId(10), navigation(1)), (CommitId(12), navigation(2))];
        assert_eq!(superseded_through(undo_then_redo.into_iter(), vouch(2)), Some((CommitId(12), CommitId(2))));
        assert_eq!(superseded_through(undo_then_redo.into_iter(), vouch(1)), None);
        assert_eq!(superseded_through([(CommitId(9), Other)].into_iter(), |_| true), None);
        assert_eq!(superseded_through([(CommitId(10), Navigation { selected: None })].into_iter(), |_| true), None);
        assert!(needs_witness(CommitId(1), Other, None, off_line));
    }

    /// A write before the latest navigation that is not on the line (made on
    /// a branch the navigation moved away from) is superseded.
    #[test]
    fn an_off_line_write_before_it_is_still_superseded() {
        let away_write_back = [(CommitId(10), navigation(1)), (CommitId(11), Other), (CommitId(12), navigation(2))];
        let superseded = superseded_through(away_write_back.into_iter(), vouch(2));
        assert!(!needs_witness(CommitId(11), Other, superseded, off_line));
    }

    /// Public #453: a write the document places on the line from the source
    /// to the selected basis is in the restored state, so it keeps needing its
    /// witness. The line is asked about the selected basis, and only for
    /// non-navigation occurrences.
    #[test]
    fn a_line_write_before_the_latest_navigation_still_needs_its_witness() {
        let kinds = [(CommitId(13), Other), (CommitId(16), Other), (CommitId(18), navigation(15))];
        let superseded = superseded_through(kinds.into_iter(), vouch(15));
        let on_line = |commit: CommitId, selected: CommitId| selected == CommitId(15) && commit == CommitId(13);
        assert!(needs_witness(CommitId(13), Other, superseded, on_line));
        assert!(!needs_witness(CommitId(16), Other, superseded, on_line));
        assert!(!needs_witness(CommitId(18), navigation(15), superseded, |_, _| true));
    }
}
