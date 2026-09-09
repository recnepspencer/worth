use std::sync::Arc;

use worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease;
use worth_runtime_bridge::facade::BridgeRetainedConditionalDecisionSeed;

use super::{WorthQueryConditionalAdmissionDenial, WorthQueryConditionalProvenance};

/// An escaped Bridge decision keeps the full product that admitted it.
pub(crate) struct WorthQueryRetainedConditionalDecision {
    seed: BridgeRetainedConditionalDecisionSeed,
    product: Arc<WorthQueryProductBranchLease>,
}

impl WorthQueryConditionalProvenance {
    pub(crate) fn retain_for_reentry(&self) -> WorthQueryRetainedConditionalDecision {
        WorthQueryRetainedConditionalDecision {
            seed: self.bridge.retain_for_reentry(),
            product: Arc::clone(&self.product),
        }
    }
}

impl WorthQueryRetainedConditionalDecision {
    pub(crate) fn admit_product(
        &self,
        selected: &WorthQueryProductBranchLease,
    ) -> Result<Arc<WorthQueryProductBranchLease>, WorthQueryConditionalAdmissionDenial> {
        if !self.product.has_same_selected_occurrence(selected) {
            return Err(WorthQueryConditionalAdmissionDenial::ProductSelectionMismatch);
        }
        Ok(Arc::clone(&self.product))
    }

    pub(crate) fn seed(&self) -> &BridgeRetainedConditionalDecisionSeed {
        &self.seed
    }
}
