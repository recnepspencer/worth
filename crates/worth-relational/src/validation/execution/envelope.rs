use crate::authority::commit::preparation::diagnostics::counters::ValidationPreparationCounters;
use crate::authority::commit::preparation::diagnostics::failures::PreparationFailureClass;
use crate::authority::commit::preparation::diagnostics::observations::ValidationDiagnosticObservation;
use crate::authority::commit::preparation::reduction::identity::ValidationResultIdentity;
use crate::authority::commit::preparation::reduction::keys::ValidationReductionKey;
use crate::validation::data::{InvariantCheckResult, InvariantReportedRule, InvariantVerdict};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvariantWorkerResult {
    pub(crate) result_identity: ValidationResultIdentity,
    pub(crate) result: InvariantCheckResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvariantWorkerEnvelope {
    pub(crate) packet_index: usize,
    pub(crate) reduction_key: ValidationReductionKey,
    pub(crate) results: Vec<InvariantWorkerResult>,
    pub(crate) diagnostic_observations: Vec<ValidationDiagnosticObservation>,
    pub(crate) preparation_failures: Vec<PreparationFailureClass>,
    pub(crate) counters: ValidationPreparationCounters,
}

impl InvariantWorkerEnvelope {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        // Read owned capacities directly: encoding just to measure them would
        // allocate outside the packet's admitted scratch allowance.
        let vector_bytes = (self.results.capacity() as u64)
            .saturating_mul(std::mem::size_of::<InvariantWorkerResult>() as u64)
            .saturating_add(
                (self.diagnostic_observations.capacity() as u64)
                    .saturating_mul(std::mem::size_of::<ValidationDiagnosticObservation>() as u64),
            )
            .saturating_add(
                (self.preparation_failures.capacity() as u64)
                    .saturating_mul(std::mem::size_of::<PreparationFailureClass>() as u64),
            );
        let result_bytes = self.results.iter().fold(0_u64, |total, worker| {
            total
                .saturating_add(check_result_bytes(&worker.result))
                .saturating_add(identity_bytes(&worker.result_identity))
        });
        let diagnostics_bytes = self
            .diagnostic_observations
            .iter()
            .fold(0_u64, |total, observation| {
                total.saturating_add(identity_bytes(&observation.result_identity))
            });
        vector_bytes
            .saturating_add(result_bytes)
            .saturating_add(diagnostics_bytes)
            .saturating_add(
                (self.reduction_key.partition_scope.len() as u64).saturating_mul(
                    std::mem::size_of::<crate::identity::data::PartitionId>() as u64,
                ),
            )
    }
}

pub(crate) fn identity_bytes(identity: &ValidationResultIdentity) -> u64 {
    rule_bytes(&identity.rule).saturating_add(identity.witness.owned_allocation_capacity_bytes())
}

fn rule_bytes(rule: &InvariantReportedRule) -> u64 {
    match rule {
        InvariantReportedRule::Native(_) => 0,
        InvariantReportedRule::Custom(identity) => identity.rule_id.as_str().len() as u64,
    }
}

pub(crate) fn check_result_bytes(result: &InvariantCheckResult) -> u64 {
    let provenance_bytes = result.custom_provenance.as_ref().map_or(0, |provenance| {
        (provenance.touched.visible_entity_ids.len() as u64)
            .saturating_mul(std::mem::size_of::<crate::identity::data::EntityId>() as u64)
            .saturating_add((provenance.touched.visible_relation_ids.len() as u64)
                .saturating_mul(std::mem::size_of::<crate::identity::data::RelationId>() as u64))
            .saturating_add((provenance.touched.touched_partition_ids.len() as u64)
                .saturating_mul(std::mem::size_of::<crate::identity::data::PartitionId>() as u64))
    });
    let verdict_bytes = match &result.verdict {
        InvariantVerdict::Pass | InvariantVerdict::NotApplicable => 0,
        InvariantVerdict::Violation(violation) | InvariantVerdict::Advisory { violation, .. } => {
            (violation.detail.capacity() as u64)
                .saturating_add(violation.fields.owned_allocation_capacity_bytes())
        }
    };
    rule_bytes(&result.rule)
        .saturating_add(result.witness.owned_allocation_capacity_bytes())
        .saturating_add(provenance_bytes)
        .saturating_add(verdict_bytes)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidationReducerConflict {
    pub(crate) identity: ValidationResultIdentity,
}
