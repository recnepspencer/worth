//! Typed stops of exact input and output verification.
use super::CompanionPreflightStop;
#[derive(Debug)]
pub enum InputCutoffVerificationStop {
    Admission(CompanionPreflightStop),
    OutputObservation(crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    WorkExhausted,
    CapacityExhausted,
    PendingUpstream,
    PendingOutput(
        std::sync::Arc<
            crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity,
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
