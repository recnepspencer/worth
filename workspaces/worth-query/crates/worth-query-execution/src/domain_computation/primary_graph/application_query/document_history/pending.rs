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
    pub(super) fn product(&self) -> &WorthQueryProductObservationLease {
        &self.product
    }
    pub(super) fn retained_source<S: ApplicationSchema>(
        &self,
        runtime: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>,
    ) -> std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
    > {
        crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation::from_product(
            runtime,
            self.product.retained_clone(),
        )
    }

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
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>,
        branch: WorthQueryProductBranch,
        value: Value,
    ) -> Result<Value, (Value, Denial)> {
        if !self.current {
            return Err((value, Denial::UnsupportedReadMode));
        }
        if self.identity().product_branch() != branch {
            return Err((value, Denial::HeadDiverged));
        }
        confirm_document_head(runtime, phase, &self.product, value)
    }

    /// Native/World checks run outside the cell. Rechecking the same physical
    /// selection afterwards prevents a still-current old branch from disclosing
    /// a result as the newly selected document. Private pending work is irrelevant.
    pub(super) fn confirm_selection<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Value,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        value: Value,
    ) -> Result<Value, Denial> {
        let branch = {
            let selected = document.selection.lock();
            if self.identity() != selected.expected {
                return Err(Denial::HeadDiverged);
            }
            selected.head.branch()
        };
        let value = self
            .confirm(phase, document.application(), branch, value)
            .map_err(|(_, denial)| denial)?;
        let selected = document.selection.lock();
        if self.identity() != selected.expected || selected.head.branch() != branch {
            return Err(Denial::HeadDiverged);
        }
        drop(selected);
        Ok(value)
    }
}
