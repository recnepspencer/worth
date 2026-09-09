use super::super::super::WorthQueryApplicationIdempotencyBinding;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationSettlementNextAction {
    RecoverDeferredApplicationSettlement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationSettlementDeferred {
    stage: crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage,
    detail: String,
    counters:
        crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters,
    settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
    publication_failure_stage:
        Option<crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage>,
    publication_failure_detail: Option<String>,
    publication_failure_counters: Option<
        crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters,
    >,
    idempotency_binding: WorthQueryApplicationIdempotencyBinding,
    branch: worth_relational::facade::history::BranchId,
    product_affinity:
        crate::domain_computation::primary_graph::provider::WorthQueryProductIdempotencyAffinity,
}

impl WorthQueryApplicationSettlementDeferred {
    pub(in crate::domain_computation::primary_graph) fn from_provider_session(
        deferred: crate::domain_computation::provider_session::WorthQueryProviderSessionSettlementDeferred,
        idempotency_binding: WorthQueryApplicationIdempotencyBinding,
        branch: worth_relational::facade::history::BranchId,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ) -> Self {
        let publication_failure = deferred.publication_failure();
        Self {
            stage: deferred.stage(),
            detail: deferred.detail().to_owned(),
            counters: deferred.counters(),
            settlement: deferred.settlement().clone(),
            publication_failure_stage: publication_failure.map(|failure| failure.stage()),
            publication_failure_detail: publication_failure
                .map(|failure| failure.detail().to_owned()),
            publication_failure_counters: publication_failure.map(|failure| failure.counters()),
            idempotency_binding,
            branch,
            product_affinity:
                crate::domain_computation::primary_graph::provider::WorthQueryProductIdempotencyAffinity::from_observation(
                    product.observation(),
                ),
        }
    }

    pub const fn stage(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters
    {
        self.counters
    }

    pub(in crate::domain_computation::primary_graph) fn settlement(
        &self,
    ) -> &worth_relational::facade::publication::DeferredPublicationSettlement {
        &self.settlement
    }

    pub const fn publication_failure_stage(
        &self,
    ) -> Option<crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage>
    {
        self.publication_failure_stage
    }

    pub fn publication_failure_detail(&self) -> Option<&str> {
        self.publication_failure_detail.as_deref()
    }

    pub const fn publication_failure_counters(
        &self,
    ) -> Option<
        crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters,
    > {
        self.publication_failure_counters
    }

    pub const fn next_action(&self) -> WorthQueryApplicationSettlementNextAction {
        WorthQueryApplicationSettlementNextAction::RecoverDeferredApplicationSettlement
    }

    pub fn commit_id(&self) -> worth_relational::facade::history::CommitId {
        self.settlement.commit().commit_id
    }

    pub(in crate::domain_computation::primary_graph) const fn requires_idempotency_readmission(
        &self,
    ) -> bool {
        self.publication_failure_detail.is_some()
    }

    pub(in crate::domain_computation::primary_graph) const fn idempotency_binding(
        &self,
    ) -> WorthQueryApplicationIdempotencyBinding {
        self.idempotency_binding
    }

    pub(in crate::domain_computation::primary_graph) fn branch(
        &self,
    ) -> &worth_relational::facade::history::BranchId {
        &self.branch
    }

    pub(in crate::domain_computation::primary_graph) fn product_affinity(
        &self,
    ) -> &crate::domain_computation::primary_graph::provider::WorthQueryProductIdempotencyAffinity
    {
        &self.product_affinity
    }
}
