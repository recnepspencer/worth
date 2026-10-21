//! Immutable resource decisions, prepared once before invocation admission.

use std::sync::Arc;

use super::*;

#[cfg(test)]
mod tests;
use crate::domain_computation::execution_resource_admission::admission_plan_digest::admitted_plan_identity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreparedExecutionResourcePlan {
    pub(super) basis: Arc<PreparedExecutionResourceBasis>,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct PreparedExecutionResourceBasis {
    pub(super) contract_identity: Arc<str>,
    pub(super) request: Arc<WorthQueryExecutionResourceRequest>,
    pub(super) request_identity: Arc<str>,
    pub(super) envelope_identity: Arc<str>,
    pub(super) envelope: Arc<WorthQueryExecutionResourceEnvelope>,
    pub(super) support_snapshot: WorthQueryExecutionResourceSupportSnapshot,
    pub(super) strategy: WorthQueryExecutionStrategyContract,
    pub(super) posture: WorthQueryExecutionResourceAdmissionPosture,
}

impl PreparedExecutionResourcePlan {
    pub(crate) fn new(
        contract_identity: String,
        request: &WorthQueryExecutionResourceRequest,
        support_snapshot: WorthQueryExecutionResourceSupportSnapshot,
        strategy: WorthQueryExecutionStrategyContract,
    ) -> Self {
        let request_identity = request.canonical_identity().into();
        let envelope_identity = admitted_envelope_identity(strategy.envelope()).into();
        let envelope = Arc::new(strategy.envelope().clone());
        let posture = if strategy.envelope().degradation().is_some() {
            WorthQueryExecutionResourceAdmissionPosture::Degraded
        } else {
            WorthQueryExecutionResourceAdmissionPosture::Exact
        };
        Self {
            basis: Arc::new(PreparedExecutionResourceBasis {
                contract_identity: contract_identity.into(),
                request: Arc::new(request.clone()),
                request_identity,
                envelope_identity,
                envelope,
                support_snapshot,
                strategy,
                posture,
            }),
        }
    }

    pub(crate) fn support(&self) -> &WorthQueryExecutionResourceSupportSnapshot {
        &self.basis.support_snapshot
    }

    pub(crate) fn bind(
        &self,
        binding_identity: &str,
        counters: WorthQueryExecutionResourceAdmissionCounters,
    ) -> WorthQueryAdmittedExecutionResourcePlan {
        WorthQueryAdmittedExecutionResourcePlan {
            identity: admitted_plan_identity(
                binding_identity,
                &self.basis.contract_identity,
                &self.basis.request_identity,
                self.basis.support_snapshot.identity(),
                self.basis.strategy.name().as_str(),
                &self.basis.envelope_identity,
            )
            .into(),
            binding_identity: binding_identity.into(),
            basis: Arc::clone(&self.basis),
            counters,
        }
    }
}
