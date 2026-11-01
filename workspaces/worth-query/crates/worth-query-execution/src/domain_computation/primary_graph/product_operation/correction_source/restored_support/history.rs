//! Canonical candidates retain their ordinary or managed selection provenance.
use super::super::WorthQueryCorrectionDerivationDenial as Denial;
use super::{
    cancellation::Occurrences, overlap::Selection,
    WorthQueryCorrectionRestoredSupportDenial as Stop,
};
use crate::domain_computation::primary_graph::fact_verification_observation::correction_rejection;
use std::num::NonZeroUsize;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    change_source::RelationalSelectedCommit,
    history::{RelationalAuthoredWriteCursor, RelationalAuthoredWritePageLimits},
    runtime::RelationalRuntime,
};

mod relevance;
pub(super) use relevance::narrow;

pub(super) struct SelectedHistory {
    ordinary: Occurrences,
    managed: Occurrences,
}

pub(super) fn collect(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    original: &AdmittedRelationalBranchBasis,
    target: &AdmittedRelationalBranchBasis,
    selection: &Selection,
) -> Result<SelectedHistory, Denial> {
    let mut ordinary = Occurrences::default();
    let mut managed = Occurrences::default();
    // Preserve the existing truly-empty selection refusal. An all-managed
    // selection alone may skip the ordinary cursor's unrelated EmptySelection.
    if !selection.keys.is_empty() || selection.managed.is_empty() {
        let cursor = runtime
            .authored_write_history_cursor(
                execution,
                original,
                selected(runtime, target)?,
                &selection.keys,
            )
            .map_err(|cause| Denial::RestoredSupport(Stop::Native(cause)))
            .inspect_err(|cause| correction_rejection("canonical-history", None, cause))?;
        pages(
            runtime,
            execution,
            cursor,
            &mut ordinary,
            "canonical-history-page",
        )?;
    }
    if !selection.managed.is_empty() {
        let cursor = runtime
            .authored_managed_source_history_cursor(
                execution,
                original,
                selected(runtime, target)?,
                &selection.managed,
            )
            .map_err(|cause| Denial::RestoredSupport(Stop::Native(cause)))
            .inspect_err(|cause| {
                correction_rejection("managed-scope-candidate-history", None, cause)
            })?;
        // Only weak metadata can be acquired under this runtime loan. Historical
        // predicate work follows outside it, before genuine cancellation.
        pages(
            runtime,
            execution,
            cursor,
            &mut managed,
            "managed-scope-candidate-page",
        )?;
    }
    Ok(SelectedHistory { ordinary, managed })
}
fn selected(
    runtime: &RelationalRuntime,
    target: &AdmittedRelationalBranchBasis,
) -> Result<RelationalSelectedCommit, Denial> {
    let commit = target
        .observation()
        .commit_id()
        .ok_or(Denial::MissingTargetParent)?;
    runtime
        .select_exact_commit(&target.observation(), commit)
        .into_outcome()
        .map_err(|cause| Denial::RestoredSupport(Stop::Selection(cause)))
}
fn pages(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    mut cursor: RelationalAuthoredWriteCursor,
    occurrences: &mut Occurrences,
    stage: &'static str,
) -> Result<(), Denial> {
    loop {
        // A physical paging granule, not a semantic history ceiling.
        let page = cursor
            .next_page(
                runtime,
                execution,
                RelationalAuthoredWritePageLimits {
                    maximum_entries: NonZeroUsize::new(32).expect("positive page width"),
                    maximum_source_visits: NonZeroUsize::new(32).expect("positive page width"),
                    maximum_owned_bytes: NonZeroUsize::new(usize::MAX)
                        .expect("positive storage bound"),
                },
            )
            .map_err(|cause| Denial::RestoredSupport(Stop::Native(cause)))
            .inspect_err(|cause| correction_rejection(stage, None, cause))?;
        for occurrence in page.entries() {
            super::ordered_visit(execution, occurrences.len()).map_err(Denial::RestoredSupport)?;
            occurrences
                .insert_cloned_native_for_request(&occurrence.commit_id(), occurrence, execution)
                .map_err(|cause| Denial::RestoredSupport(Stop::Storage(cause)))?;
        }
        if page.is_complete() {
            return Ok(());
        }
    }
}
