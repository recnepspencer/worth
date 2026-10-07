//! One current-output denial rule for verification and evidence retention.

use super::*;

pub(super) fn from_stop(
    stop: ConsumedOutputVerificationStop,
    subject: &str,
) -> WorthQueryCurrentOutputDenial {
    WorthQueryCurrentOutputDenial::new(
        match stop {
            ConsumedOutputVerificationStop::WorkExhausted => {
                WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded
            }
            ConsumedOutputVerificationStop::CapacityExhausted => {
                WorthQueryCurrentOutputDenialKind::RetentionCapacityExhausted
            }
            ConsumedOutputVerificationStop::PendingUpstream => {
                WorthQueryCurrentOutputDenialKind::PendingUpstream
            }
            ConsumedOutputVerificationStop::RetryCurrentness(stop) => {
                WorthQueryCurrentOutputDenialKind::CurrentnessRaced(stop)
            }
            ConsumedOutputVerificationStop::Unavailable => {
                WorthQueryCurrentOutputDenialKind::OutputUnavailable
            }
            ConsumedOutputVerificationStop::Interrupted(event) => {
                WorthQueryCurrentOutputDenialKind::Interrupted(event.interruption())
            }
        },
        subject,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop;
    use worth_relational::facade::mvcc::CompanionPreflightStop;

    #[test]
    fn a_full_ledger_reports_capacity_to_the_handler_not_output_unavailable() {
        let stop = CompanionPreflightStop::RetainedCompanionCapacityExhausted {
            requested: 1,
            retained: 1,
            maximum: 1,
        };
        let denied = from_stop(map_admission_stop(stop), "consumed-family");
        assert_eq!(
            denied.kind(),
            WorthQueryCurrentOutputDenialKind::RetentionCapacityExhausted
        );
        assert_eq!(denied.subject(), "consumed-family");
    }
}
