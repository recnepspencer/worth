use worth_runtime_bridge::facade::{
    BridgeManagedClockObservationOutcome, BridgeManagedTemporalDenialKind,
};

use super::{ErasedClockObservationOutcome, ErasedClockObservationReceipt};

pub(super) fn map_bridge_clock_outcome(
    outcome: Result<
        BridgeManagedClockObservationOutcome,
        worth_runtime_bridge::facade::BridgeManagedTemporalDenial,
    >,
    retain: impl FnOnce(
        worth_runtime_bridge::facade::BridgeManagedClockAcceptedObservation,
    ) -> ErasedClockObservationReceipt,
) -> ErasedClockObservationOutcome {
    match outcome {
        Ok(BridgeManagedClockObservationOutcome::Accepted(accepted)) => {
            ErasedClockObservationOutcome::Accepted(retain(accepted))
        }
        Ok(BridgeManagedClockObservationOutcome::Duplicate(duplicate)) => {
            ErasedClockObservationOutcome::Duplicate(retain(duplicate))
        }
        Ok(BridgeManagedClockObservationOutcome::Stale) => ErasedClockObservationOutcome::Stale,
        Ok(BridgeManagedClockObservationOutcome::Reordered) => {
            ErasedClockObservationOutcome::Reordered
        }
        Err(denial) => map_bridge_denial(denial.kind(), denial.detail().to_string()),
    }
}

fn map_bridge_denial(
    kind: BridgeManagedTemporalDenialKind,
    detail: String,
) -> ErasedClockObservationOutcome {
    match kind {
        BridgeManagedTemporalDenialKind::ClosedClockBinding => {
            ErasedClockObservationOutcome::Closed
        }
        BridgeManagedTemporalDenialKind::RetentionCapacityExhausted => {
            ErasedClockObservationOutcome::Failed {
                kind: super::WorthQueryConditionalClockObservationFailureKind::RetentionCapacityExhausted,
                detail,
            }
        }
        _ => super::authoritative_clock_progression::runtime_rejection(detail),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_retention_capacity_exhausted_preserves_typed_failure_kind() {
        let outcome = map_bridge_denial(
            BridgeManagedTemporalDenialKind::RetentionCapacityExhausted,
            "retained slot ceiling reached".to_string(),
        );
        assert!(matches!(
            outcome,
            ErasedClockObservationOutcome::Failed {
                kind: super::super::WorthQueryConditionalClockObservationFailureKind::RetentionCapacityExhausted,
                ..
            }
        ));
        assert!(matches!(
            map_bridge_denial(
                BridgeManagedTemporalDenialKind::ClosedClockBinding,
                "closed".to_string(),
            ),
            ErasedClockObservationOutcome::Closed
        ));
    }
}
