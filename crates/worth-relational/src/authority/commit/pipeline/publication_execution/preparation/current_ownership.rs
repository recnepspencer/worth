//! Select current custody once in the commit's original work account.
use crate::{
    branch::AdmittedRelationalBranchBasis,
    publication::ownership::{self, CurrentPublicationRoster},
    runtime::RelationalPreparationRuntime,
    transactions::data::TransactionCommitError,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(super) fn select(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    candidate: &crate::history::data::CanonicalCommitEnvelope,
    adjacency: &[crate::authority::mutation::AdjacencyDelta],
    working: &crate::storage::overlay::WorkingState,
    admission: Option<&crate::mvcc::RequiredOutputPublicationAdmission>,
) -> Result<Option<Arc<CurrentPublicationRoster>>, TransactionCommitError> {
    if basis
        .inner
        .root
        .schema_authority()
        .registry()
        .current_publication_storage
        .is_none()
    {
        return Ok(None);
    }
    let targets = candidate.patch.authoritative_record_patches.len();
    let bytes = targets
        .checked_mul(4 * std::mem::size_of::<crate::transactions::data::RecordRef>())
        .and_then(|n| {
            adjacency
                .len()
                .checked_mul(4 * std::mem::size_of::<crate::identity::data::RelationId>())
                .and_then(|extra| n.checked_add(extra))
        })
        .ok_or_else(|| {
            ownership::request(
                crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
            )
        })?;
    let _selection_funding = execution
        .reserve_retained_memory(bytes as u64)
        .map_err(ownership::request)?;
    let retargeted = adjacency
        .iter()
        .map(|delta| delta.relation_id)
        .collect::<std::collections::HashSet<_>>();
    let mut records = candidate
        .patch
        .authoritative_record_patches
        .iter()
        .filter(|p| {
            p.structural_change != crate::publication::patch::data::RecordStructuralChange::Created
        })
        .map(|p| p.target.clone())
        .collect::<Vec<_>>();
    let mut scopes = candidate
        .patch
        .authoritative_record_patches
        .iter()
        .filter(|p| match p.structural_change {
            crate::publication::patch::data::RecordStructuralChange::Created => false,
            crate::publication::patch::data::RecordStructuralChange::Updated => {
                matches!(p.target, crate::transactions::data::RecordRef::Relation(id)
                    if retargeted.contains(&id))
            }
            _ => true,
        })
        .map(|p| p.target.clone())
        .collect::<Vec<_>>();
    if let Some(admission) = admission {
        admission.publication.ownership_seeds(&mut scopes);
    }
    for event in &candidate.required_output_codec_retirements {
        scopes.push(crate::transactions::data::RecordRef::Entity(
            event.meaning.source_scope,
        ));
    }
    records.extend(scopes.iter().cloned());
    let after_image =
        crate::storage::overlay::OverlayStateView::new(basis.inner.root.as_ref(), working);
    let after = crate::validation::engine::state_view::InvariantStateView::new(
        &after_image,
        candidate.commit.version_id,
    );
    let run = || {
        let destroyed = candidate
            .patch
            .authoritative_record_patches
            .iter()
            .filter(|p| {
                matches!(p.structural_change,
                crate::publication::patch::data::RecordStructuralChange::Deleted |
                crate::publication::patch::data::RecordStructuralChange::RetainedForAudit |
                crate::publication::patch::data::RecordStructuralChange::MaterializationSuspended)
            })
            .map(|p| p.target.clone())
            .collect::<Vec<_>>();
        execution
            .run_native_with_checkpoint(|context| {
                ownership::checkpoint(context, (targets + adjacency.len()) as u64)?;
                let mut linked =
                    ownership::linked_selectors(execution, basis, &destroyed, context)?;
                let mut scoped = linked.clone();
                scoped.extend(scopes.iter().cloned());
                linked.extend(records.iter().cloned());
                // Retained admission evidence comes from complete source-scope
                // readmission. Locally assembled cascade predecessors stay within
                // this preparation and never feed this reuse decision.
                if let Some(current) =
                    admission.and_then(|a| a.publication.current_ownership_evidence())
                    && current.basis.as_ref() == Some(basis.descriptor())
                    && current.covers(&linked, &scoped)
                {
                    return Ok(Arc::clone(current));
                }
                ownership::read_selected(execution, basis, &linked, &scoped, Some(&after), context)
                    .map(Arc::new)
            })
            .map_err(ownership::request)?
            .0
    };
    let result = match (
        super::super::super::request_lease(execution),
        runtime.commit_work_budget.as_ref(),
    ) {
        (Some(_), Some(budget)) => budget
            .run_stage(*execution, run)
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?,
        _ => run(),
    }?;
    Ok(Some(result))
}
