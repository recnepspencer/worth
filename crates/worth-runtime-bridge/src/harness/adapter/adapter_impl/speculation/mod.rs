use super::*;

mod churn;
mod churn_certification;
mod discard;
mod promotion;
mod shared;
pub(in crate::harness::adapter::adapter_impl) mod terminal_report_export;

#[cfg(test)]
mod typed_certification_tests;

pub(super) enum SpeculationHarnessTarget {
    DiscardCertification,
    PromotionCertification,
    ChurnCertification,
}

pub(super) enum SpeculationHarnessExecution {
    Discard {
        execution_record: crate::facade::BridgePreviewExecutionRecord,
        discard_record: crate::facade::BridgePreviewDiscardRecord,
        routing_digest: Option<String>,
    },
    Promotion {
        promoted_execution_record: crate::facade::BridgePreviewExecutionRecord,
        promotion_record: crate::facade::BridgePreviewPromotionRecord,
        promoted_replay_bundle: crate::facade::BridgePreviewReplayBundle,
        discarded_execution_record: crate::facade::BridgePreviewExecutionRecord,
        discarded_record: crate::facade::BridgePreviewDiscardRecord,
        discarded_replay_bundle: crate::facade::BridgePreviewReplayBundle,
        routing_digest: Option<String>,
        diagnostics_digest: String,
    },
    Churn {
        certification: churn_certification::SpeculationChurnCertification,
    },
}

pub(super) fn execute_speculation_request(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &crate::harness::fixtures::BridgeHarnessFixture,
    target: SpeculationHarnessTarget,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<SpeculationHarnessExecution, BridgeHarnessError> {
    match target {
        SpeculationHarnessTarget::DiscardCertification => {
            discard::execute_discard_certification(runtime_bridge, fixture, resource_request)
        }
        SpeculationHarnessTarget::PromotionCertification => {
            promotion::execute_promotion_certification(runtime_bridge, fixture, resource_request)
        }
        SpeculationHarnessTarget::ChurnCertification => {
            churn::execute_churn_certification(runtime_bridge, fixture, resource_request)
        }
    }
}
