//! Exact-commit evidence struct and read-only accessors.

pub mod postcommit_currentness;

pub use postcommit_currentness::{
    FactlessCurrentness, FailedRebase, OwnEffect, OwnEffectOnReads, PreparedRebaseDenial,
    RebaseVerificationFailure, RebaseVerificationReason, RebasedSourceFacts,
};

use crate::domain_computation::primary_graph::provider::{
    mutation_work::WorthQueryPrimaryMutationWorkEvidence,
    session_commit::WorthQueryCommittedDispatchOutboxResolution,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphCommitEvidence {
    pub workflow_origin: Option<crate::domain_computation::primary_graph::workflow::operation_output::WorkflowOperationOrigin>,
    pub provider_session_binding:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    pub idempotency:
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
    pub commit: worth_relational::facade::history::RelationalCommitReceipt,
    pub mutation_work: WorthQueryPrimaryMutationWorkEvidence,
    pub retained_preimage:
        Option<crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage>,
    pub original_authority: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding,
    >,
    pub committed_dispatch_outbox: WorthQueryCommittedDispatchOutboxResolution,
    pub committed_changes: crate::domain_computation::primary_graph::WorthQueryApplicationCommittedChanges,
    pub output_correspondence: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    >,
    pub operation_scope: crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
    pub observed_source_facts: RebasedSourceFacts,
    pub performed_input_facts: Option<crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload>,
    pub performed_output_fields: Option<Result<crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload, crate::domain_computation::primary_graph::application_attempt::OutputFieldCaptureFailure>>,
}

impl WorthQueryPrimaryGraphCommitEvidence {
    pub fn performed_output_fields(
        &self,
    ) -> Option<&Result<crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload, crate::domain_computation::primary_graph::application_attempt::OutputFieldCaptureFailure>>{
        self.performed_output_fields.as_ref()
    }

    pub fn performed_input_facts(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload>
    {
        self.performed_input_facts.as_ref()
    }

    pub fn workflow_origin(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::workflow::operation_output::WorkflowOperationOrigin>{
        self.workflow_origin.as_ref()
    }

    pub fn retain_original_authority(
        &self,
    ) -> std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding,
    > {
        std::sync::Arc::clone(&self.original_authority)
    }

    pub fn original_authority(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding
    {
        &self.original_authority
    }

    pub const fn provider_session_binding(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding
    {
        &self.provider_session_binding
    }

    pub const fn idempotency(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding{
        self.idempotency
    }

    pub const fn commit_reference(
        &self,
    ) -> &worth_relational::facade::history::RelationalCommitReceipt {
        &self.commit
    }

    pub const fn mutation_work(&self) -> &WorthQueryPrimaryMutationWorkEvidence {
        &self.mutation_work
    }

    pub const fn retained_preimage(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        self.retained_preimage.as_ref()
    }

    pub const fn committed_dispatch_outbox(&self) -> &WorthQueryCommittedDispatchOutboxResolution {
        &self.committed_dispatch_outbox
    }

    pub const fn committed_changes(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationCommittedChanges {
        &self.committed_changes
    }

    pub fn output_correspondence(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence {
        self.output_correspondence.as_ref()
    }

    pub fn retain_output_correspondence(
        &self,
    ) -> std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    > {
        std::sync::Arc::clone(&self.output_correspondence)
    }

    pub const fn operation_scope(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeBinding {
        &self.operation_scope
    }

    pub fn source_request_stop(
        &self,
    ) -> Option<worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        match self.observed_source_facts.retained() {
            Err(FailedRebase { reason, .. }) => match reason.into_reason() {
                RebaseVerificationReason::Execution(cause) => Some(cause),
                _ => None,
            },
            Ok(_) => None,
        }
    }

    pub fn posting_continuation(
        &self,
    ) -> Option<
        crate::domain_computation::primary_graph::output_lineage::invalidation::PostingContinuation,
    > {
        self.observed_source_facts.posting_continuation()
    }

    pub fn source_payload_bytes(&self) -> Option<u64> {
        self.observed_source_facts.payload_bytes()
    }

    pub fn rebased_source_facts(
        &self,
    ) -> Result<
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        FailedRebase,
    > {
        self.observed_source_facts.retained()
    }

    pub fn source_facts_superseded_by_own_effect(&self) -> impl Iterator<Item = usize> + '_ {
        self.observed_source_facts.superseded_by_own_effect()
    }
}

mod court_doors_for_test;
