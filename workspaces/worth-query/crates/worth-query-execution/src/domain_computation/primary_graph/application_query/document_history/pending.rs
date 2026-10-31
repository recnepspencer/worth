use super::*;
use crate::basis::{
    WorthQueryProductBranch, WorthQueryProductBranchReadIdentity, WorthQueryProductObservationLease,
};
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryModelIdentity;

pub(super) struct PendingDocumentRead {
    product: WorthQueryProductObservationLease,
    model: HistoryModelIdentity,
    current: bool,
}

impl PendingDocumentRead {
    pub(super) fn capture<S, Q, P, R, A, I, E>(
        plan: &WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, E>,
    ) -> Self {
        Self {
            product: plan.basis.retained_product(),
            model: HistoryModelIdentity::from_resolved(plan.scope),
            current: plan.controls.is_current_product_one_shot(),
        }
    }

    pub(super) fn model(&self) -> HistoryModelIdentity {
        self.model.clone()
    }

    pub(super) fn identity(&self) -> WorthQueryProductBranchReadIdentity {
        WorthQueryProductBranchReadIdentity::from_observation(self.product.observation())
    }

    pub(super) fn confirm<S: ApplicationSchema, Value>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        branch: WorthQueryProductBranch,
        value: Value,
    ) -> Result<Value, (Value, Denial)> {
        if !self.current {
            return Err((value, Denial::UnsupportedReadMode));
        }
        if self.identity().product_branch() != branch {
            return Err((value, Denial::HeadDiverged));
        }
        current_head::confirm(runtime, phase, &self.product, value)
    }
}
