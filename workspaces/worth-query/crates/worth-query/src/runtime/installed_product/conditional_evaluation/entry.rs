use std::sync::Arc;

use worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease;
use worth_runtime_bridge::facade::{
    BridgeConditionalDenial, BridgeConditionalEvaluationAdmissionRequest,
    BridgeConditionalEvaluationSession, BridgeConditionalExecutionRequest,
    BridgeInstalledConditionalLowering,
};

use super::{
    cache::WorthQueryIdleCacheEntry, preparation::WorthQueryPreparationCell,
    WorthQueryExecutedConditional,
};
use crate::runtime::installed_product::WorthQueryInstalledProduct;

pub(super) struct WorthQueryConditionalEvaluationEntry {
    product: Arc<WorthQueryProductBranchLease>,
    lowering: Arc<BridgeInstalledConditionalLowering>,
    session: WorthQueryPreparationCell<BridgeConditionalEvaluationSession>,
}

impl WorthQueryConditionalEvaluationEntry {
    pub(super) fn new(
        product: &Arc<WorthQueryProductBranchLease>,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
    ) -> Self {
        Self {
            product: Arc::clone(product),
            lowering: Arc::clone(lowering),
            session: Default::default(),
        }
    }

    pub(super) fn execute(
        &self,
        session: &BridgeConditionalEvaluationSession,
        installed: &WorthQueryInstalledProduct,
        request: BridgeConditionalExecutionRequest<'_>,
        context: &mut dyn std::any::Any,
    ) -> Result<WorthQueryExecutedConditional, BridgeConditionalDenial> {
        let request = BridgeConditionalExecutionRequest {
            bridge_snapshot_identity: (self.lowering.correspondence_count() != 0)
                .then(|| self.product.bridge_snapshot_identity()),
            ..request
        };
        let decision = installed
            .conditional
            .execute_admitted_conditional(session, request, context)?;
        Ok(WorthQueryExecutedConditional {
            decision,
            product: Arc::clone(&self.product),
        })
    }

    pub(super) fn admit_session(
        &self,
        installed: &WorthQueryInstalledProduct,
    ) -> Result<Arc<BridgeConditionalEvaluationSession>, BridgeConditionalDenial> {
        self.session.get_or_try_init(|| self.admit(installed))
    }

    pub(super) fn matches(
        &self,
        product: &WorthQueryProductBranchLease,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
    ) -> bool {
        self.product.has_same_selected_occurrence(product) && Arc::ptr_eq(&self.lowering, lowering)
    }

    fn admit(
        &self,
        installed: &WorthQueryInstalledProduct,
    ) -> Result<BridgeConditionalEvaluationSession, BridgeConditionalDenial> {
        let signal_basis = self
            .product
            .admit_conditional_signal_basis(&installed.conditional, &self.lowering)?;
        let request = if self.lowering.correspondence_count() == 0 {
            BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(&signal_basis)
        } else {
            BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                &signal_basis,
                self.product.bridge_snapshot_identity(),
            )
        };
        installed.conditional.admit_conditional_evaluation(request)
    }
}

impl WorthQueryIdleCacheEntry for WorthQueryConditionalEvaluationEntry {
    fn is_idle(entry: &Arc<Self>) -> bool {
        Arc::strong_count(entry) == 1
    }
}
