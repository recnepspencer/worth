use super::*;
use crate::harness::fixtures::BridgeHarnessFixture;

mod certification_digest_basis;
mod certification_execution;
mod matrices;
mod shared_artifacts;
pub(in crate::harness::adapter::adapter_impl) mod terminal_report_export;

use matrices::{PolicyCertificationMatrix, RequestPolicyMatrix, RoutePolicyMatrix};
#[cfg(test)]
mod typed_certification_tests;

pub(super) enum PolicyHarnessTarget {
    ProvenanceCertification,
    RejectionCertification,
    AmbientLeakCertification,
}

pub(super) enum PolicyHarnessExecution {
    Provenance {
        policy_digest: String,
        policy_matrix: PolicyCertificationMatrix,
        policy_provenance_report: crate::facade::BridgePolicyProvenanceReport,
        request_policy_matrix: RequestPolicyMatrix,
        route_policy_matrix: RoutePolicyMatrix,
        routing_digest: Option<String>,
        replay_digest: String,
        diagnostics_digest: String,
        counter_snapshot: crate::facade::BridgePolicyCounters,
    },
    Rejection {
        policy_matrix: PolicyCertificationMatrix,
        failure_digest: String,
        diagnostics_digest: String,
        counter_snapshot: crate::facade::BridgePolicyCounters,
    },
    AmbientLeak {
        policy_digest: String,
        policy_matrix: PolicyCertificationMatrix,
        policy_provenance_report: crate::facade::BridgePolicyProvenanceReport,
        request_policy_matrix: RequestPolicyMatrix,
        replay_digest: String,
        diagnostics_digest: String,
        counter_snapshot: crate::facade::BridgePolicyCounters,
    },
}

pub(super) fn execute_policy_request(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    target: PolicyHarnessTarget,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<PolicyHarnessExecution, BridgeHarnessError> {
    match target {
        PolicyHarnessTarget::ProvenanceCertification => {
            certification_execution::execute_provenance_certification(
                runtime_bridge,
                fixture,
                resource_request,
            )
        }
        PolicyHarnessTarget::RejectionCertification => {
            certification_execution::execute_rejection_certification(runtime_bridge)
        }
        PolicyHarnessTarget::AmbientLeakCertification => {
            certification_execution::execute_ambient_leak_certification(runtime_bridge, fixture)
        }
    }
}
