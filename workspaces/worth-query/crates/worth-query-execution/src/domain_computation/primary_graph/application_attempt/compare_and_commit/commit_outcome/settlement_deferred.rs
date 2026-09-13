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
        Self {
            stage: deferred.stage(),
            detail: deferred.detail().to_owned(),
            counters: deferred.counters(),
            settlement: deferred.settlement().clone(),
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

    pub const fn next_action(&self) -> WorthQueryApplicationSettlementNextAction {
        WorthQueryApplicationSettlementNextAction::RecoverDeferredApplicationSettlement
    }

    /// Relational owner fact retained for settlement diagnostics. This is not
    /// a product commit and cannot authorize dispatch or current-state reads.
    pub fn relational_settlement_commit(
        &self,
    ) -> &worth_relational::facade::history::RelationalCommitReceipt {
        self.settlement.commit()
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
