//! Selected-index admission preserves every resource and ordinary refusal.
use super::{
    entity_denial, WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind as Kind,
};
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as Refusal;
use worth_relational::facade::{
    mvcc::{CompanionPreflightStop as Stop, RelationalOperationInterruption},
    transactions::RelationalExecutionDenialCause as Native,
};
pub(super) fn denial(stop: Stop, subject: &str) -> WorthQueryEntityResolutionDenial {
    let kind = match stop {
        Stop::Execution(cause) => Kind::Stopped(cause),
        Stop::Interrupted(event) => Kind::Stopped(match event.interruption() {
            RelationalOperationInterruption::Cancelled => Native::Cancelled,
            RelationalOperationInterruption::TimedOut => Native::DeadlineElapsed,
        }),
        Stop::WorkExhausted { .. } | Stop::WorkExhaustedWithoutRequirement { .. } => {
            Kind::Stopped(Native::WorkExhausted)
        }
        Stop::WorkCounterOverflow => Kind::Stopped(Native::WorkCounterOverflow),
        Stop::PreparationMemoryCounterOverflow | Stop::RetainedCompanionCounterOverflow { .. } => {
            Kind::Stopped(Native::ChargedBytesOverflow)
        }
        Stop::PreparationMemoryExhausted { required, maximum } => {
            Kind::Stopped(Native::DeclaredMemoryExhausted {
                requested: required,
                admitted: maximum,
            })
        }
        Stop::RetainedCompanionCapacityExhausted {
            requested,
            retained,
            maximum,
        } => Kind::Stopped(match requested.checked_add(retained) {
            Some(requested) => Native::DeclaredMemoryExhausted {
                requested,
                admitted: maximum,
            },
            None => Native::ChargedBytesOverflow,
        }),
        Stop::TopologyPending
        | Stop::SelectedPositionUnavailable { .. }
        | Stop::RegistrationChanged => Kind::Refused(Refusal::EqualityIndexUnavailable),
        Stop::SelectedSourceMismatch | Stop::ForeignCell => {
            Kind::Refused(Refusal::ForeignResolutionTruth)
        }
    };
    entity_denial(kind, subject)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_counter_overflow_stays_distinct_from_a_finite_resolution_ceiling() {
        let subject = "original selected identity field";
        let overflow = denial(
            Stop::RetainedCompanionCounterOverflow {
                requested: 1,
                retained: u64::MAX,
            },
            subject,
        );
        assert_eq!(overflow.kind(), Kind::Stopped(Native::ChargedBytesOverflow));
        assert_eq!(overflow.subject(), subject);
        let finite = denial(
            Stop::RetainedCompanionCapacityExhausted {
                requested: 3,
                retained: 1,
                maximum: 2,
            },
            subject,
        );
        assert_eq!(
            finite.kind(),
            Kind::Stopped(Native::DeclaredMemoryExhausted {
                requested: 4,
                admitted: 2,
            })
        );
        assert_eq!(finite.subject(), subject);
    }
}
