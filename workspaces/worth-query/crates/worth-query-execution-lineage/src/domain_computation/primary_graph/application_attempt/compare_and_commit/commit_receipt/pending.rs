use super::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationCommitTerminalEvidence,
    WorthQueryCommittedAftermathCausality, WorthQueryCommittedReceiptProjection,
    WorthQueryMutationPreconditionComparisonEvidence, WorthQueryRetainedPreImage,
};

pub struct WorthQueryPendingApplicationCommitReceipt {
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
    pub fn original_provider(&self) -> &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication{
        &self.provider
    }
    pub fn original_runtime_instance_id(&self) -> u64 {
        self.provider.runtime_instance_id()
    }
    pub fn original_commit(
        &self,
    ) -> &worth_relational::facade::history::RelationalCommitReceipt {
        self.provider.commit_reference()
    }
    pub fn original_publication(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication {
        self.provider.committed_product_publication()
    }
    pub fn original_work(
        &self,
    ) -> Option<
        &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryMutationWorkEvidence,
    > {
        self.provider.mutation_work()
    }
    pub fn original_preimage(
        &self,
    ) -> Option<&WorthQueryRetainedPreImage> {
        self.retained_preimage.as_ref()
    }
    pub fn original_outbox_binding(&self) -> Option<&crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxBinding>{
        self.committed_dispatch_outbox.as_ref()
    }
    pub fn original_authority(
        &self,
    ) -> &super::WorthQueryApplicationCommitAuthorityBinding {
        &self.authority_binding
    }
    pub fn canonical_work(
        &self,
    ) -> worth_query_installation::facade::WorthQueryCanonicalWorkPhases {
        self.canonical_work
    }
    pub fn dispatch_binding(&self) -> Option<&crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxBinding>{
        self.committed_dispatch_outbox.as_ref()
    }

    pub fn from_projection(
        current_provider_session: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
        projection: WorthQueryCommittedReceiptProjection,
        precondition_comparison: WorthQueryMutationPreconditionComparisonEvidence,
        canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
        authority_binding: super::WorthQueryApplicationCommitAuthorityBinding,
    ) -> Option<Self> {
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

    pub fn with_aftermath_causality(
        mut self,
        causality: Option<WorthQueryCommittedAftermathCausality>,
    ) -> Self {
        self.aftermath_causality = causality;
        self
    }

    pub fn complete(
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
        let performed_product_change = if self.provider.is_coupled_member() {
            None
        } else {
            Some(self.provider.take_fresh_product_change()?)
        };
        let receipt = WorthQueryApplicationCommitReceipt {
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
            performed_product_change,
            committed_changes: self.provider.commit_evidence().committed_changes().clone(),
            performed_output_fields: self
                .provider
                .commit_evidence()
                .performed_output_fields()
                .cloned(),
            performed_input_facts: self
                .provider
                .commit_evidence()
                .performed_input_facts()
                .cloned(),
            output_correspondence: self
                .provider
                .commit_evidence()
                .retain_output_correspondence(),
            exact_output_settlement: self.provider.exact_output_settlement().cloned(),
            unrebased_own_effect: self
                .provider
                .commit_evidence()
                .rebased_source_facts()
                .err()
                .map(|failed| failed.own_effect),
        };
        super::recording::observe(&receipt);
        Some(receipt)
    }
}
