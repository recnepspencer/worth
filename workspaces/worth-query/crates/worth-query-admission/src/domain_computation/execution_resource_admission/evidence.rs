use std::collections::BTreeMap;
use std::sync::Arc;

use worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest;
use worth_query_installation::facade::{
    WorthQueryExecutionResourceEnvelope, WorthQueryExecutionStrategyContract,
    WorthQueryExecutionStrategyName,
};

use crate::admission_digest::hash_parts;

use super::admission_plan_digest::admitted_envelope_identity;
use super::WorthQueryExecutionResourceSupportSnapshot;

mod prepared;
use prepared::PreparedExecutionResourceBasis;
pub(crate) use prepared::{PreparedExecutionResourcePlan, PreparedResourcePlanAdmissionStop};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryExecutionResourceAdmissionCounters {
    pub runtime_authority_checks: usize,
    pub input_contract_checks: usize,
    pub execution_contract_checks: usize,
    pub resource_contract_lookups: usize,
    pub support_snapshot_checks: usize,
    pub strategy_checks: usize,
    pub envelope_dimension_checks: usize,
    pub capacity_reservation_checks: usize,
    pub capacity_reservations: usize,
    pub provider_session_mints: usize,
}

impl WorthQueryExecutionResourceAdmissionCounters {
    fn accumulate(&mut self, other: Self) {
        self.runtime_authority_checks += other.runtime_authority_checks;
        self.input_contract_checks += other.input_contract_checks;
        self.execution_contract_checks += other.execution_contract_checks;
        self.resource_contract_lookups += other.resource_contract_lookups;
        self.support_snapshot_checks += other.support_snapshot_checks;
        self.strategy_checks += other.strategy_checks;
        self.envelope_dimension_checks += other.envelope_dimension_checks;
        self.capacity_reservation_checks += other.capacity_reservation_checks;
        self.capacity_reservations += other.capacity_reservations;
        self.provider_session_mints += other.provider_session_mints;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExecutionResourceAdmissionPosture {
    Exact,
    Degraded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedExecutionResourcePlan {
    identity: Arc<str>,
    binding_identity: Arc<str>,
    basis: Arc<PreparedExecutionResourceBasis>,
    counters: WorthQueryExecutionResourceAdmissionCounters,
}

impl WorthQueryAdmittedExecutionResourcePlan {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    pub fn contract_identity(&self) -> &str {
        &self.basis.contract_identity
    }

    pub fn request_identity(&self) -> &str {
        &self.basis.request_identity
    }

    pub fn request(&self) -> &WorthQueryExecutionResourceRequest {
        &self.basis.request
    }

    pub fn envelope_identity(&self) -> &str {
        &self.basis.envelope_identity
    }

    pub fn support_snapshot(&self) -> &WorthQueryExecutionResourceSupportSnapshot {
        &self.basis.support_snapshot
    }

    pub fn strategy(&self) -> &WorthQueryExecutionStrategyName {
        self.basis.strategy.name()
    }

    pub fn envelope(&self) -> &WorthQueryExecutionResourceEnvelope {
        &self.basis.envelope
    }

    pub fn posture(&self) -> WorthQueryExecutionResourceAdmissionPosture {
        self.basis.posture
    }

    pub fn counters(&self) -> WorthQueryExecutionResourceAdmissionCounters {
        self.counters
    }

    pub fn record_provider_session_mint(&mut self) {
        self.counters.provider_session_mints += 1;
    }

    pub fn record_capacity_reservation_check(&mut self) {
        self.counters.capacity_reservation_checks += 1;
    }

    pub fn record_capacity_reservation(&mut self) {
        self.counters.capacity_reservations += 1;
    }

    pub fn shared_envelope(&self) -> Arc<WorthQueryExecutionResourceEnvelope> {
        Arc::clone(&self.basis.envelope)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedWorkflowResourcePlan {
    operation: WorthQueryAdmittedExecutionResourcePlan,
    stages: BTreeMap<String, Arc<WorthQueryAdmittedExecutionResourcePlan>>,
    identity: Arc<str>,
    counters: WorthQueryExecutionResourceAdmissionCounters,
}

impl WorthQueryAdmittedWorkflowResourcePlan {
    pub fn assemble(
        operation: WorthQueryAdmittedExecutionResourcePlan,
        stages: BTreeMap<String, WorthQueryAdmittedExecutionResourcePlan>,
    ) -> Self {
        let stages = stages
            .into_iter()
            .map(|(identity, plan)| (identity, Arc::new(plan)))
            .collect::<BTreeMap<_, _>>();
        let mut counters = operation.counters();
        for plan in stages.values() {
            counters.accumulate(plan.counters());
        }
        let identity = Arc::<str>::from(hash_parts(&[
            "worth_query_admitted_workflow_resource_plan_v1".into(),
            format!("operation:{}", operation.identity()),
            format!(
                "stages:{}",
                stages
                    .iter()
                    .map(|(stage, plan)| format!("{stage}:{}", plan.identity()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ]));
        Self {
            operation,
            stages,
            identity,
            counters,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn operation(&self) -> &WorthQueryAdmittedExecutionResourcePlan {
        &self.operation
    }

    pub fn counters(&self) -> WorthQueryExecutionResourceAdmissionCounters {
        self.counters
    }

    pub fn record_provider_session_mint(&mut self) {
        self.operation.record_provider_session_mint();
        self.counters.provider_session_mints += 1;
    }

    pub fn record_capacity_reservation_check(&mut self) {
        self.operation.record_capacity_reservation_check();
        self.counters.capacity_reservation_checks += 1;
    }

    pub fn record_capacity_reservation(&mut self) {
        self.operation.record_capacity_reservation();
        self.counters.capacity_reservations += 1;
    }

    pub fn stages(&self) -> impl Iterator<Item = (&str, &WorthQueryAdmittedExecutionResourcePlan)> {
        self.stages
            .iter()
            .map(|(identity, plan)| (identity.as_str(), plan.as_ref()))
    }

    pub fn stage(&self, identity: &str) -> Option<&WorthQueryAdmittedExecutionResourcePlan> {
        self.stages.get(identity).map(Arc::as_ref)
    }

    pub fn shared_stage(
        &self,
        identity: &str,
    ) -> Option<Arc<WorthQueryAdmittedExecutionResourcePlan>> {
        self.stages.get(identity).map(Arc::clone)
    }
}
