use super::super::{
    WorthQueryCorrectionDerivationDenial as Denial, WorthQuerySelectedCorrectionSource,
    WorthQuerySelectedProductOperation,
};
use super::{cancellation, overlap, WorthQueryCorrectionRestoredSupportDenial as Stop};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use crate::domain_computation::primary_graph::fact_verification_observation::correction_rejection;
use crate::domain_computation::primary_graph::{
    exact_basis_access::open_exact_basis_snapshot,
    output_reuse::{first_moved_fact, retained_fact_denial},
    WorthQueryAdvancementPhase,
};
use std::num::NonZeroUsize;
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    history::RelationalAuthoredWritePageLimits, runtime::RelationalRuntime,
};

#[cfg(test)]
mod retirement;

pub(in crate::domain_computation::primary_graph::product_operation::correction_source) fn require<
    Schema: ApplicationSchema,
>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    facts: &FactPayload,
) -> Result<(), Denial> {
    let application = target.application();
    let execution = phase
        .execution_request_for(&application.product_runtime)
        .map_err(|_| Denial::ForeignPhase)?;
    // Direct-head correction preserves its existing strict comparison for ALL
    // fact classes. Equality is the actual World occurrence AND admitted basis,
    // never merely a Native root, field revision or value equality.
    if target.product().observation().selected_commit() == source.ancestry().source().identity()
        && target.product().observation().basis() == source.ancestry().source().basis()
    {
        return application
            .primary_provider
            .graph
            .with_runtime_mut(&execution, |runtime| {
                let snapshot =
                    open_exact_basis_snapshot(runtime, target.product().relational_basis())
                        .map_err(Denial::Basis)?;
                let result = strict(
                    runtime,
                    execution,
                    &snapshot,
                    facts,
                    "exact source occurrence",
                );
                crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
                result
            });
    }
    let original = source.ancestry().source().basis().relational_basis();
    let selection = application
        .primary_provider
        .graph
        .with_runtime_mut(&execution, |runtime| {
            with_support_snapshots(
                runtime,
                original,
                target.product().relational_basis(),
                |runtime, before, current| {
                    let original_view = runtime
                        .read_truth()
                        .project_snapshot(before)
                        .ok_or(Denial::SemanticMaterialUnavailable)?;
                    let target_view = runtime
                        .read_truth()
                        .project_snapshot(current)
                        .ok_or(Denial::SemanticMaterialUnavailable)?;
                    overlap::select(execution, facts, &original_view, &target_view)
                        .map_err(Denial::RestoredSupport)
                        .inspect_err(|cause| correction_rejection("overlap-selection", None, cause))
                },
            )
        })?;
    let occurrences =
        application
            .primary_provider
            .graph
            .with_runtime_mut(&execution, |runtime| {
                let basis = target.product().relational_basis();
                let commit = basis
                    .observation()
                    .commit_id()
                    .ok_or(Denial::MissingTargetParent)?;
                let selected = runtime
                    .select_exact_commit(&basis.observation(), commit)
                    .into_outcome()
                    .map_err(|cause| Denial::RestoredSupport(Stop::Selection(cause)))?;
                let mut cursor = runtime
                    .authored_write_history_cursor(execution, original, selected, &selection.keys)
                    .map_err(|cause| Denial::RestoredSupport(Stop::Native(cause)))
                    .inspect_err(|cause| correction_rejection("canonical-history", None, cause))?;
                let mut occurrences = cancellation::Occurrences::default();
                loop {
                    // A physical paging granule, not a semantic history ceiling.
                    let page = cursor
                        .next_page(
                            runtime,
                            execution,
                            RelationalAuthoredWritePageLimits {
                                maximum_entries: NonZeroUsize::new(32)
                                    .expect("positive page width"),
                                maximum_source_visits: NonZeroUsize::new(32)
                                    .expect("positive page width"),
                                maximum_owned_bytes: NonZeroUsize::new(usize::MAX)
                                    .expect("positive storage bound"),
                            },
                        )
                        .map_err(|cause| Denial::RestoredSupport(Stop::Native(cause)))
                        .inspect_err(|cause| {
                            correction_rejection("canonical-history-page", None, cause)
                        })?;
                    for occurrence in page.entries() {
                        // The occurrence shares admitted canonical material; only these
                        // new immutable cells are quoted. No receipt payload is cloned.
                        super::ordered_visit(execution, occurrences.len())
                            .map_err(Denial::RestoredSupport)?;
                        occurrences
                            .insert_cloned_native_for_request(
                                &occurrence.commit_id(),
                                occurrence,
                                execution,
                            )
                            .map_err(|cause| Denial::RestoredSupport(Stop::Storage(cause)))?;
                    }
                    if page.is_complete() {
                        break;
                    }
                }
                Ok::<_, Denial>(occurrences)
            })?;
    cancellation::require(target, execution, &occurrences)
        .inspect_err(|cause| correction_rejection("canonical-cancellation", None, cause))?;
    application
        .primary_provider
        .graph
        .with_runtime_mut(&execution, |runtime| {
            with_support_snapshots(
                runtime,
                original,
                target.product().relational_basis(),
                |runtime, before, current| compare(runtime, execution, before, current, facts),
            )
        })
}

fn with_support_snapshots<T>(
    runtime: &mut RelationalRuntime,
    original: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    target: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    inspect: impl FnOnce(
        &RelationalRuntime,
        &worth_relational::facade::snapshots::SnapshotHandle,
        &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<T, Denial>,
) -> Result<T, Denial> {
    let before = open_exact_basis_snapshot(runtime, original).map_err(Denial::Basis)?;
    let result = match open_exact_basis_snapshot(runtime, target) {
        Ok(current) => {
            let result = inspect(runtime, &before, &current);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &current);
            result
        }
        Err(cause) => Err(Denial::Basis(cause)),
    };
    crate::relational_snapshot_release::release_query_snapshot(runtime, &before);
    result
}

fn compare(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    original: &worth_relational::facade::snapshots::SnapshotHandle,
    target: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: &FactPayload,
) -> Result<(), Denial> {
    strict(
        runtime,
        execution,
        original,
        facts,
        "original correction support",
    )?;
    let original_view = runtime
        .read_truth()
        .project_snapshot(original)
        .ok_or(Denial::SemanticMaterialUnavailable)?;
    let target_view = runtime
        .read_truth()
        .project_snapshot(target)
        .ok_or(Denial::SemanticMaterialUnavailable)?;
    for fact in facts.iter() {
        compare_fact(
            runtime,
            execution,
            target,
            &original_view,
            &target_view,
            fact,
        )
        .inspect_err(|cause| {
            correction_rejection("paired-support-comparison", Some(fact), cause)
        })?;
    }
    Ok(())
}

fn compare_fact(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    target: &worth_relational::facade::snapshots::SnapshotHandle,
    original_view: &worth_relational::facade::runtime::VisibilityProjectionView,
    target_view: &worth_relational::facade::runtime::VisibilityProjectionView,
    fact: &Fact,
) -> Result<(), Denial> {
    match fact {
        Fact::SourceAspectRevision {
            entity_id, aspect, ..
        } => {
            if !original_view
                .compare_entity_aspect_support(&execution, target_view, *entity_id, aspect)
                .map_err(|cause| Denial::RestoredSupport(Stop::FieldSupport(cause)))?
            {
                return Err(Denial::SourceDiverged);
            }
        }
        Fact::SourceAdjacencyRevision {
            relation_kind,
            anchor,
            direction,
            ..
        } => {
            if !original_view
                .compare_adjacency_support(
                    &execution,
                    target_view,
                    *anchor,
                    *relation_kind,
                    *direction,
                )
                .map_err(|cause| Denial::RestoredSupport(Stop::Adjacency(cause)))?
            {
                return Err(Denial::SourceDiverged);
            }
        }
        Fact::SourceFieldRevision {
            entity_id, locator, ..
        } => {
            if !original_view
                .compare_entity_field_support(&execution, target_view, *entity_id, locator)
                .map_err(|cause| Denial::RestoredSupport(Stop::FieldSupport(cause)))?
            {
                return Err(Denial::SourceDiverged);
            }
        }
        _ => match first_moved_fact(&execution, runtime, target, std::iter::once(fact)) {
            Ok(None) => {}
            Ok(Some(_)) => return Err(Denial::SourceDiverged),
            Err(cause) => {
                return Err(Denial::DependencyComparison(retained_fact_denial(
                    cause,
                    "restored correction support",
                )))
            }
        },
    }
    Ok(())
}

fn strict(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: &FactPayload,
    subject: &'static str,
) -> Result<(), Denial> {
    match first_moved_fact(&execution, runtime, snapshot, facts.iter()) {
        Ok(None) => Ok(()),
        Ok(Some(fact)) => {
            correction_rejection(subject, Some(fact), &Denial::SourceDiverged);
            Err(Denial::SourceDiverged)
        }
        Err(cause) => {
            let fact = match &cause {
                crate::domain_computation::primary_graph::output_reuse::RetainedFactStop::Unavailable(fact) => Some(*fact),
                _ => None,
            };
            let denial = Denial::DependencyComparison(retained_fact_denial(cause, subject));
            correction_rejection(subject, fact, &denial);
            Err(denial)
        }
    }
}
