use super::{
    WorthQueryApplicationCommitReceipt, WorthQueryCommittedReceiptProjection,
    WorthQueryMutationPreconditionComparisonEvidence,
};

impl WorthQueryApplicationCommitReceipt {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn from_idempotency_read(
        _permit: super::super::super::idempotency_resolution::WorthQueryIdempotencyReadCommitReceiptPermit,
        projection: WorthQueryCommittedReceiptProjection,
        precondition_comparison: WorthQueryMutationPreconditionComparisonEvidence,
        canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
        authority_binding: super::WorthQueryApplicationCommitAuthorityBinding,
    ) -> Self {
        Self::from_recovered_projection(
            projection,
            precondition_comparison,
            canonical_work,
            authority_binding,
            None,
        )
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn from_early_equivalent(
        _permit: super::super::super::provider_execution::WorthQueryEarlyEquivalentCommitReceiptPermit,
        projection: WorthQueryCommittedReceiptProjection,
        precondition_comparison: WorthQueryMutationPreconditionComparisonEvidence,
        canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
        authority_binding: super::WorthQueryApplicationCommitAuthorityBinding,
    ) -> Self {
        Self::from_recovered_projection(
            projection,
            precondition_comparison,
            canonical_work,
            authority_binding,
            None,
        )
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn from_stale_equivalent(
        permit: super::super::super::provider_execution::WorthQueryStaleEquivalentCommitReceiptPermit,
        projection: WorthQueryCommittedReceiptProjection,
        precondition_comparison: WorthQueryMutationPreconditionComparisonEvidence,
        canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
        authority_binding: super::WorthQueryApplicationCommitAuthorityBinding,
    ) -> Self {
        Self::from_recovered_projection(
            projection,
            precondition_comparison,
            canonical_work,
            authority_binding,
            Some(permit.into_provider_session()),
        )
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn from_managed_equivalent(
        permit: super::super::super::provider_execution::WorthQueryManagedEquivalentCommitReceiptPermit,
        projection: WorthQueryCommittedReceiptProjection,
        precondition_comparison: WorthQueryMutationPreconditionComparisonEvidence,
        canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
        authority_binding: super::WorthQueryApplicationCommitAuthorityBinding,
    ) -> Self {
        Self::from_recovered_projection(
            projection,
            precondition_comparison,
            canonical_work,
            authority_binding,
            Some(permit.into_provider_session()),
        )
    }

    fn from_recovered_projection(
        projection: WorthQueryCommittedReceiptProjection,
        precondition_comparison: WorthQueryMutationPreconditionComparisonEvidence,
        canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
        authority_binding: super::WorthQueryApplicationCommitAuthorityBinding,
        expected_retry_session: Option<
            crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
        >,
    ) -> Self {
        let (provider, committed_dispatch_outbox) = projection.into_parts();
        let authoritative_provider_session = provider
            .commit_evidence()
            .provider_session_binding()
            .clone();
        let retained_preimage = provider.commit_evidence().retained_preimage().cloned();
        let terminal = super::WorthQueryApplicationCommitTerminalEvidence::recovered(
            provider.branch().clone(),
        );
        Self {
            authoritative_provider_session,
            outcome_identity: provider.application_outcome_identity(),
            provider_runtime_instance_id: provider.runtime_instance_id(),
            commit: provider.commit_reference().clone(),
            basis_descriptor: provider.basis_descriptor().clone(),
            changed_record_count: provider.changed_record_count(),
            emitted_effect_count: provider.emitted_effect_count(),
            mutation_work: provider.mutation_work().cloned(),
            precondition_comparison,
            canonical_work,
            terminal,
            committed_dispatch_outbox,
            external_dispatch: None,
            external_dispatch_preparation_denial: None,
            authority_binding,
            retained_preimage,
            aftermath_causality: None,
            expected_retry_session,
            performed_product_change: None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn with_retry_cleanup(
        mut self,
        completion: crate::domain_computation::provider_session::WorthQueryMutationGraphWorkCompletion,
    ) -> Option<Self> {
        let expected = self.expected_retry_session.take()?;
        let actual = completion.provider_session_binding()?;
        if !expected.same_session(actual)
            || completion.relational_branch() != self.terminal.branch()
        {
            return None;
        }
        self.terminal = self
            .terminal
            .with_retry_cleanup(completion.attempt_resources_released());
        Some(self)
    }
}
