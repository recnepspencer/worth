//! Native-owned inverse preparation for one exact recorded cascade retirement.
use super::*;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::CanonicalCommitEnvelope,
    identity::data::{EntityId, RelationId},
    transactions::data::*,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct RelationalAuthoredRestoration {
    pub(crate) before: AdmittedRelationalBranchBasis,
    pub(crate) retirement: Arc<CanonicalCommitEnvelope>,
    pub(crate) anchor: CreatedEntityRef,
    pub(crate) entities: Vec<(EntityId, CreatedEntityRef)>,
    pub(crate) relations: Vec<(RelationId, CreatedRelationRef)>,
    pub(crate) publications: Vec<RelationalRequiredOutputPublication>,
    pub(crate) inverse: Vec<MutationIntent>,
    pub(crate) content: crate::content::RelationalContentAttachments,
    pub(crate) aggregate_memberships: Vec<crate::aggregate_identity::RestoredMembership>,
}
impl std::fmt::Debug for RelationalAuthoredRestoration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalAuthoredRestoration")
            .field("retirement", &self.retirement.commit)
            .field("anchor", &self.anchor)
            .finish_non_exhaustive()
    }
}
impl RelationalAuthoredRestoration {
    /// Exact retained content for the Native-owned inverse records.
    pub fn content(&self) -> &crate::content::RelationalContentAttachments {
        &self.content
    }
    pub fn authorities(
        &self,
    ) -> impl Iterator<Item = &worth_proof::RequiredOutputPublicationAuthority> {
        self.publications.iter().map(|prior| prior.authority())
    }
    /// Append Native's recorded inverse before ordinary invariant validation.
    pub fn extend_batch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut batch: WorkerIntentBatch,
    ) -> Result<WorkerIntentBatch, TransactionCommitError> {
        let result = execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    for intent in &self.inverse {
                        context.checkpoint(
                            intent.owned_allocation_capacity_bytes().saturating_add(1),
                        )?;
                        batch = batch.push(intent.clone());
                    }
                    Ok::<_, crate::execution::RelationalExecutionDenialCause>(batch)
                })
        });
        execution
            .consult()
            .map_err(super::custody::execution_stop)?;
        result
            .map_err(super::custody::execution_stop)?
            .map_err(super::custody::execution_stop)?
            .0
            .map_err(super::custody::execution_stop)
    }
}
#[cfg(test)]
#[path = "authored_restoration_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
