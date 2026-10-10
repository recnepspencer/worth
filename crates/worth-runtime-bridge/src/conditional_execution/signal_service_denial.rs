use super::{BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeSignalDenial};
pub(super) fn signal_service_denial(
    error: worth_signal::facade::branch::SignalConditionalServiceExecutionDenial,
) -> BridgeConditionalDenial {
    use worth_signal::facade::branch::SignalConditionalServiceExecutionDenial as Denial;
    let detail = format!("Signal conditional service denied execution: {error:?}");
    let kind = match error {
        Denial::SlotAdmission(ref cause) => super::signal_execution_denial::kind(cause),
        native @ (Denial::OwnerUnavailable(_)
        | Denial::OwnerAdmission(_)
        | Denial::StaleBasisAdmission
        | Denial::DefinitionReadmissionRequired
        | Denial::DefinitionMismatch
        | Denial::NestedOperationScopeMismatch
        | Denial::MissingSourceEvidence
        | Denial::UnexpectedSourceEvidence
        | Denial::SourceAuthorityMismatch
        | Denial::EvaluationIdentityExhausted
        | Denial::AdmissionCapacityExhausted
        | Denial::AdmissionUnavailable
        | Denial::SlotBusy
        | Denial::SlotPoisoned
        | Denial::ObservationAdmission(_)
        | Denial::UnconsumedUnwind) => BridgeConditionalDenialKind::SignalExecution(
            BridgeSignalDenial::ConditionalExecution(native),
        ),
    };
    BridgeConditionalDenial::new(kind, detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_signal::facade::branch::SignalConditionalServiceExecutionDenial as Native;

    #[test]
    fn native_conditional_cause_survives_bridge_admission() {
        for native in [
            Native::SlotBusy,
            Native::DefinitionMismatch,
            Native::StaleBasisAdmission,
        ] {
            assert_eq!(
                signal_service_denial(native.clone()).kind(),
                BridgeConditionalDenialKind::SignalExecution(
                    BridgeSignalDenial::ConditionalExecution(native)
                ),
            );
        }
    }

    #[test]
    fn native_slot_cancellation_keeps_the_request_control_cause() {
        let native = Native::SlotAdmission(
            worth_signal::facade::SignalError::ExecutionCheckpointStopped(
                worth_signal::facade::SignalCheckpointDenial::Cancelled,
            ),
        );
        assert_eq!(
            signal_service_denial(native).kind(),
            BridgeConditionalDenialKind::ExecutionDenied(
                crate::error::BridgeExecutionDenial::Cancelled
            )
        );
    }
}
