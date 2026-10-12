//! Typed stops of exact input and output verification.
use super::CompanionPreflightStop;
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum InputCutoffVerificationStop {
    Admission(CompanionPreflightStop),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    WorkExhausted,
    CapacityExhausted,
    PendingUpstream,
    PendingOutput(
        std::sync::Arc<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >,
    ),
    CurrentnessRaced,
    SelectedSourceUnavailable,
    SelectedSourceMismatch,
}

impl From<CompanionPreflightStop> for InputCutoffVerificationStop {
    fn from(stop: CompanionPreflightStop) -> Self {
        Self::Admission(stop)
    }
}
