use super::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationCommitTerminalEvidence,
    WorthQueryCommittedAftermathCausality, WorthQueryCommittedReceiptProjection,
    WorthQueryMutationPreconditionComparisonEvidence, WorthQueryRetainedPreImage,
};

pub(in crate::domain_computation) struct WorthQueryPendingApplicationCommitReceipt {
    provider: super::super::super::super::provider::WorthQueryPrimaryGraphCommittedApplication,
    current_provider_session:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    precondition_comparison: WorthQueryMutationPreconditionComparisonEvidence,
    canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
    committed_dispatch_outbox:
        Option<super::super::super::super::provider::WorthQueryCommittedDispatchOutboxBinding>,
    authority_binding: super::WorthQueryApplicationCommitAuthorityBinding,
    retained_preimage: Option<WorthQueryRetainedPreImage>,
    aftermath_causality: Option<WorthQueryCommittedAftermathCausality>,
}

impl WorthQueryPendingApplicationCommitReceipt {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn from_projection(
        permit: super::super::super::provider_execution::WorthQueryFreshCommitReceiptPermit,
        projection: WorthQueryCommittedReceiptProjection,
        precondition_comparison: WorthQueryMutationPreconditionComparisonEvidence,
        canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
        authority_binding: super::WorthQueryApplicationCommitAuthorityBinding,
    ) -> Option<Self> {
        let current_provider_session = permit.into_provider_session();
        let (provider, committed_dispatch_outbox) = projection.into_parts();
        if !provider
            .commit_evidence()
            .provider_session_binding()
            .same_session(&current_provider_session)
        {
            return None;
        }
        let retained_preimage = provider.commit_evidence().retained_preimage().cloned();
        Some(Self {
            provider,
            current_provider_session,
            precondition_comparison,
            canonical_work,
            committed_dispatch_outbox,
            authority_binding,
            retained_preimage,
            aftermath_causality: None,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn with_aftermath_causality(
        mut self,
        causality: Option<WorthQueryCommittedAftermathCausality>,
    ) -> Self {
        self.aftermath_causality = causality;
        self
    }

    pub(in crate::domain_computation::primary_graph) fn complete(
        self,
        completion: crate::domain_computation::provider_session::WorthQueryMutationGraphWorkCompletion,
    ) -> Option<WorthQueryApplicationCommitReceipt> {
        let completion_provider_session = completion.provider_session_binding()?;
        if self.provider.branch() != completion.relational_branch()
            || !self
                .current_provider_session
                .same_session(completion_provider_session)
        {
            return None;
        }
        let mutation_work = self.provider.mutation_work()?.clone();
        let performed_product_change = self.provider.take_fresh_product_change()?;
        Some(WorthQueryApplicationCommitReceipt {
            authoritative_provider_session: self
                .provider
                .commit_evidence()
                .provider_session_binding()
                .clone(),
            outcome_identity: self.provider.application_outcome_identity(),
            provider_runtime_instance_id: self.provider.runtime_instance_id(),
            commit: self.provider.commit_reference().clone(),
            committed_product_publication: self.provider.committed_product_publication().clone(),
            basis_descriptor: self.provider.basis_descriptor().clone(),
            changed_record_count: self.provider.changed_record_count(),
            emitted_effect_count: self.provider.emitted_effect_count(),
            mutation_work: Some(mutation_work),
            precondition_comparison: self.precondition_comparison,
            canonical_work: self.canonical_work,
            terminal: WorthQueryApplicationCommitTerminalEvidence::executed(
                completion.relational_branch().clone(),
                completion.attempt_resources_released(),
            ),
            committed_dispatch_outbox: self.committed_dispatch_outbox,
            external_dispatch: None,
            external_dispatch_preparation_denial: None,
            authority_binding: self.authority_binding,
            retained_preimage: self.retained_preimage,
            aftermath_causality: self.aftermath_causality,
            expected_retry_session: None,
            performed_product_change: Some(performed_product_change),
        })
    }
}
