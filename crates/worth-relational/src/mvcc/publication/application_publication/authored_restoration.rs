//! Native-owned inverse preparation for one exact recorded cascade retirement.
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::EntityId;
use crate::identity::data::RelationId;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::{
    CreatedEntityRef, CreatedRelationRef, WorkerIntentBatch,
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
impl<'restoration> From<&'restoration RelationalAuthoredRestoration>
    for crate::history::data::canonical_commit_envelope::required_output_origin::restoration_candidate::AuthoredRestorationFacts<'restoration>
{
    fn from(restoration: &'restoration RelationalAuthoredRestoration) -> Self {
        Self {
            retirement: &restoration.retirement,
            inverse: &restoration.inverse,
            entities: &restoration.entities,
            relations: &restoration.relations,
        }
    }
}

impl<'restoration> From<&'restoration Arc<RelationalAuthoredRestoration>>
    for crate::history::data::canonical_commit_envelope::required_output_origin::restoration_candidate::AuthoredRestorationFacts<'restoration>
{
    fn from(restoration: &'restoration Arc<RelationalAuthoredRestoration>) -> Self {
        restoration.as_ref().into()
    }
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
        execution
            .run_native_with_checkpoint(|context| {
                for intent in &self.inverse {
                    crate::execution::request_stage::checkpoint(
                        context,
                        intent.owned_allocation_capacity_bytes().saturating_add(1),
                    )
                    .map_err(super::custody::execution_stop)?;
                    batch = batch.push(intent.clone());
                }
                Ok(batch)
            })
            .map_err(super::custody::execution_stop)?
            .0
    }
}
#[cfg(test)]
#[path = "authored_restoration_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
