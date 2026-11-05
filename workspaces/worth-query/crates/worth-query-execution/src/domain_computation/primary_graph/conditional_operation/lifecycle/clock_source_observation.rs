use worth_query_installation::facade::WorthQueryNamedClockFailureKind;

use super::{ErasedClockObservationOutcome, WorthQueryConditionalClockObservationFailureKind};

pub fn isolate_clock_source<Observation>(
    observe: impl FnOnce() -> Result<
        Observation,
        worth_query_installation::facade::WorthQueryNamedClockFailure,
    >,
) -> Result<Observation, ErasedClockObservationOutcome> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(observe)) {
        Ok(Ok(observation)) => Ok(observation),
        Ok(Err(failure)) => Err(match failure.kind() {
            WorthQueryNamedClockFailureKind::SourceClosed => ErasedClockObservationOutcome::Closed,
            WorthQueryNamedClockFailureKind::SourceUnavailable => {
                ErasedClockObservationOutcome::Failed {
                    kind: WorthQueryConditionalClockObservationFailureKind::SourceUnavailable,
                    detail: failure.detail().to_string(),
                }
            }
            WorthQueryNamedClockFailureKind::ObservationFailed => {
                ErasedClockObservationOutcome::Failed {
                    kind: WorthQueryConditionalClockObservationFailureKind::ObservationFailed,
                    detail: failure.detail().to_string(),
                }
            }
        }),
        Err(_) => Err(ErasedClockObservationOutcome::Failed {
            kind: WorthQueryConditionalClockObservationFailureKind::SourcePanicked,
            detail: "installed named clock source panicked".to_string(),
        }),
    }
}
