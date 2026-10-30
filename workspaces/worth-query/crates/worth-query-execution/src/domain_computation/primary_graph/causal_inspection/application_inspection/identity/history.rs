//! Canonical owner outcomes preserve actual capacity causes and Native history facts.
use super::super::history::{
    HistoryDetail, WorthQueryConditionalHistoryAvailability as Availability,
    WorthQueryConditionalHistoryDecisionClass as Class,
};
use super::*;
pub(super) struct AvailabilityParts {
    pub(super) label: &'static str,
    pub(super) first: u64,
    pub(super) second: u64,
    pub(super) level: &'static str,
    pub(super) ancestor: u64,
}
pub(super) fn availability_parts(value: Option<Availability>) -> AvailabilityParts {
    let mut parts = AvailabilityParts {
        label: "original_observation_not_retained",
        first: 0,
        second: 0,
        level: "none",
        ancestor: 0,
    };
    parts.label = match value {
        None => parts.label,
        Some(Availability::RetainedAvailable) => "retained_available",
        Some(Availability::OmittedByTier) => "omitted_by_tier",
        Some(Availability::EvictedByRetention) => "evicted_by_retention",
        Some(Availability::OwnerClosed) => "owner_closed",
        Some(Availability::CaptureDenied(cause)) => {
            let level = match cause {
                worth_execution::MemoryLimitDenial::Capacity {
                    requested,
                    admitted,
                    level,
                } => {
                    parts.first = requested;
                    parts.second = admitted;
                    level
                }
                worth_execution::MemoryLimitDenial::CounterOverflow {
                    requested,
                    charged,
                    level,
                } => {
                    parts.first = requested;
                    parts.second = charged;
                    level
                }
            };
            parts.level = match level {
                worth_execution::MemoryLimitLevel::Policy { ancestor } => {
                    parts.ancestor = u64::from(ancestor);
                    "policy"
                }
                worth_execution::MemoryLimitLevel::Process => "process",
                worth_execution::MemoryLimitLevel::Declared => "declared",
            };
            match cause {
                worth_execution::MemoryLimitDenial::Capacity { .. } => "capture_denied_capacity",
                worth_execution::MemoryLimitDenial::CounterOverflow { .. } => {
                    "capture_denied_counter_overflow"
                }
            }
        }
    };
    parts
}
pub(super) fn history_identity<Stop>(
    reference: &str,
    requested: bool,
    history: &HistoryDetail,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<BridgeIdentityEvidence, InspectionIdentityStop<Stop>> {
    let cause = match history {
        HistoryDetail::Unavailable(cause) => Some(*cause),
        HistoryDetail::Provided(_) => Some(Availability::RetainedAvailable),
        _ => None,
    };
    let availability = availability_parts(cause);
    let outcome = match history {
        HistoryDetail::NotRequested => "not_requested",
        HistoryDetail::Unselected => "not_selected",
        HistoryDetail::NotRetained => "original_observation_not_retained",
        HistoryDetail::Unavailable(_) => "owner_unavailable",
        HistoryDetail::Provided(_) => "provided",
    };
    let (attempt, before, after, class, lowering, signal) = match history {
        HistoryDetail::Provided(view) => (
            view.attempt(),
            view.output_version_before(),
            view.output_version(),
            match view.decision_class() {
                Class::ComputedChanged => "computed_changed",
                Class::ComputedRevertedClean => "computed_reverted_clean",
                Class::DependencyUnchanged => "dependency_unchanged",
                Class::SuppressedBeforeCompute => "suppressed_before_compute",
                Class::DeferredByCondition => "deferred_by_condition",
                Class::DeferredTemporal => "deferred_temporal",
                Class::DeferredOnDemand => "deferred_on_demand",
            },
            view.native().lowering_projection().label().as_ref(),
            view.native().signal_projection().label().as_ref(),
        ),
        _ => (0, 0, 0, "none", "none", "none"),
    };
    compose(
        "query.application-inspection.conditional-history.v1",
        &[
            Part::Text(reference),
            Part::Text(if requested {
                "requested"
            } else {
                "not_requested"
            }),
            Part::Text(outcome),
            Part::Text(availability.label),
            Part::Number(availability.first),
            Part::Number(availability.second),
            Part::Text(availability.level),
            Part::Number(availability.ancestor),
            Part::Text(class),
            Part::Number(attempt),
            Part::Number(before),
            Part::Number(after),
            Part::Text(lowering),
            Part::Text(signal),
        ],
        admit,
    )
}
