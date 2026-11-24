//! Native-owned inverse preparation for one exact recorded cascade retirement.
use super::*;
use contracts::identity::data::record_ids::EntityId;
use contracts::identity::data::record_ids::RelationId;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntentOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::CreatedEntityRef;
use contracts::transactions::data::primitives::CreatedRelationRef;
use contracts::transactions::data::primitives::WorkerIntentBatch;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_contracts::internal_port as contracts;

#[derive(Clone)]
pub struct RelationalAuthoredRestoration {
    pub(crate) before: AdmittedRelationalBranchBasis,
    pub(crate) retirement: Arc<CanonicalCommitEnvelope>,
    pub(crate) anchor: CreatedEntityRef,
    pub(crate) entities: Vec<(EntityId, CreatedEntityRef)>,
    pub(crate) relations: Vec<(RelationId, CreatedRelationRef)>,
    pub(crate) publications: Vec<RelationalRequiredOutputPublication>,
    pub(crate) inverse: Vec<MutationIntent>,
    pub(crate) content: worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
    pub(crate) aggregate_memberships: Vec<worth_relational_mutation::internal_port::aggregate_identity::restoration::RestoredMembership>,
}
impl<'restoration> From<&'restoration RelationalAuthoredRestoration>
    for worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::restoration_candidate::AuthoredRestorationFacts<'restoration>
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
    pub fn content(
        &self,
    ) -> &worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments
    {
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
                    contracts::execution::request_stage::checkpoint(
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

#[doc(hidden)]
pub trait RelationalAuthoredRestorationFieldsPort: Sized {
    fn before(&self) -> &AdmittedRelationalBranchBasis;
    fn entities(&self) -> &Vec<(EntityId, CreatedEntityRef)>;
    fn publications(&self) -> &Vec<RelationalRequiredOutputPublication>;
    fn relations(&self) -> &Vec<(RelationId, CreatedRelationRef)>;
    fn retirement(&self) -> &Arc<CanonicalCommitEnvelope>;
}

impl RelationalAuthoredRestorationFieldsPort for RelationalAuthoredRestoration {
    fn before(&self) -> &AdmittedRelationalBranchBasis {
        &self.before
    }
    fn entities(&self) -> &Vec<(EntityId, CreatedEntityRef)> {
        &self.entities
    }
    fn publications(&self) -> &Vec<RelationalRequiredOutputPublication> {
        &self.publications
    }
    fn relations(&self) -> &Vec<(RelationId, CreatedRelationRef)> {
        &self.relations
    }
    fn retirement(&self) -> &Arc<CanonicalCommitEnvelope> {
        &self.retirement
    }
}

#[doc(hidden)]
pub trait RelationalAuthoredRestorationOwnerPort: Sized {
    fn from_inverse(
        before: AdmittedRelationalBranchBasis,
        retirement: Arc<CanonicalCommitEnvelope>,
        anchor: CreatedEntityRef,
        entities: Vec<(EntityId, CreatedEntityRef)>,
        relations: Vec<(RelationId, CreatedRelationRef)>,
        publications: Vec<RelationalRequiredOutputPublication>,
        inverse: Vec<MutationIntent>,
        content: worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
        aggregate_memberships: Vec<worth_relational_mutation::internal_port::aggregate_identity::restoration::RestoredMembership>,
    ) -> Self;
}

impl RelationalAuthoredRestorationOwnerPort for RelationalAuthoredRestoration {
    fn from_inverse(
        before: AdmittedRelationalBranchBasis,
        retirement: Arc<CanonicalCommitEnvelope>,
        anchor: CreatedEntityRef,
        entities: Vec<(EntityId, CreatedEntityRef)>,
        relations: Vec<(RelationId, CreatedRelationRef)>,
        publications: Vec<RelationalRequiredOutputPublication>,
        inverse: Vec<MutationIntent>,
        content: worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
        aggregate_memberships: Vec<worth_relational_mutation::internal_port::aggregate_identity::restoration::RestoredMembership>,
    ) -> Self {
        Self {
            before,
            retirement,
            anchor,
            entities,
            relations,
            publications,
            inverse,
            content,
            aggregate_memberships,
        }
    }
}

#[cfg(test)]
#[path = "authored_restoration_inline_type_budget.rs"]
pub mod inline_type_budget;
