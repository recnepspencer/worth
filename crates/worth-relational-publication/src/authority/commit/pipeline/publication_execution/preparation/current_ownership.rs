//! Select current custody once in the commit's original work account.
use crate::publication::ownership;
use contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_branch::internal_port::publication::ownership::read::roster::CurrentPublicationRoster;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn select(
    execution: &ExecutionRequest<'_, '_>,
    work: Option<&contracts::execution::request_work::RequestWorkBudget>,
    basis: &AdmittedRelationalBranchBasis,
    candidate: &worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody,
    created: &contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    adjacency: &[contracts::authority::mutation::adjacency_delta::AdjacencyDelta],
    working: &worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    admission: Option<
        &crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission,
    >,
) -> Result<Option<Arc<CurrentPublicationRoster>>, TransactionCommitError> {
    if (*basis
        .owner_inner()
        .root
        .schema_authority()
        .registry()
        .owner_current_publication_storage())
    .is_none()
    {
        return Ok(None);
    }
    let targets = candidate.patch.authoritative_record_patches.len();
    let bytes = targets
        .checked_mul(4 * std::mem::size_of::<contracts::transactions::data::primitives::RecordRef>())
        .and_then(|n| {
            adjacency
                .len()
                .checked_mul(4 * std::mem::size_of::<contracts::identity::data::record_ids::RelationId>())
                .and_then(|extra| n.checked_add(extra))
        })
        .ok_or_else(|| {
            ownership::request(
                contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
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
            p.structural_change != contracts::publication::patch::data::record_structural_change::RecordStructuralChange::Created
        })
        .map(|p| p.target.clone())
        .collect::<Vec<_>>();
    let mut scopes = candidate
        .patch
        .authoritative_record_patches
        .iter()
        .filter(|p| match p.structural_change {
            contracts::publication::patch::data::record_structural_change::RecordStructuralChange::Created => false,
            contracts::publication::patch::data::record_structural_change::RecordStructuralChange::Updated => {
                matches!(p.target, contracts::transactions::data::primitives::RecordRef::Relation(id)
                    if retargeted.contains(&id))
            }
            _ => true,
        })
        .map(|p| p.target.clone())
        .collect::<Vec<_>>();
    if let Some(admission) = admission {
        admission.publication.ownership_seeds(&mut scopes);
    }
    for event in candidate.owner_required_output_codec_retirements() {
        scopes.push(
            contracts::transactions::data::primitives::RecordRef::Entity(
                event.meaning.source_scope,
            ),
        );
    }
    let after_image =
        worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView::new(
            basis.owner_inner().root.as_ref(),
            working,
        );
    let after =
        worth_relational_branch::internal_port::branch::structural_state::StructuralStateView::new(
            &after_image,
            candidate.commit.version_id,
        );
    let mut run = || {
        let destroyed = candidate
            .patch
            .authoritative_record_patches
            .iter()
            .filter(|p| {
                matches!(p.structural_change,
                contracts::publication::patch::data::record_structural_change::RecordStructuralChange::Deleted |
                contracts::publication::patch::data::record_structural_change::RecordStructuralChange::RetainedForAudit |
                contracts::publication::patch::data::record_structural_change::RecordStructuralChange::MaterializationSuspended)
            })
            .map(|p| p.target.clone())
            .collect::<Vec<_>>();
        execution
            .run_native_with_checkpoint(|context| {
                ownership::checkpoint(context, (targets + adjacency.len()) as u64)?;
                // Only a coupled group admits sources allocated in its D.
                // Preserve the existing C-only path and work for other origins.
                if admission.is_some_and(|admission| {
                    matches!(
                        admission.publication,
                        crate::mvcc::publication::application_publication::RequiredOutputPublication::CoupledAuthored { .. }
                    )
                }) {
                    // A historical Created patch alone is insufficient proof.
                    retain_existing_scopes(&mut scopes, created, context)?;
                }
                records.extend(scopes.iter().cloned());
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
        contracts::execution::request_custody::request_lease(execution),
        work,
    ) {
        (Some(_), Some(budget)) => budget
            .run_stage(*execution, run)
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?,
        _ => run(),
    }?;
    Ok(Some(result))
}

fn retain_existing_scopes(
    scopes: &mut Vec<contracts::transactions::data::primitives::RecordRef>,
    created: &contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    let mut kept = 0;
    for next in 0..scopes.len() {
        if let contracts::transactions::data::primitives::RecordRef::Entity(id) = scopes[next]
            && created
                .created_here(id, context)
                .map_err(ownership::request)?
        {
            continue;
        }
        scopes.swap(kept, next);
        kept += 1;
    }
    scopes.truncate(kept);
    Ok(())
}
