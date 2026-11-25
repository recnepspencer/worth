//! Higher-level desktop temporal contribution, the stock recurring rule
//! ([`crate::facade::worth_query_recurring_rule!`]) and bounded caller-pumped
//! reopen catch-up.

pub use crate::temporal_contribution::{
    install_desktop_temporal_clock_operation, install_desktop_temporal_operation,
    pump_temporal_catch_up, pump_temporal_catch_up_observing, pump_temporal_catch_up_until,
    pump_temporal_catch_up_until_observing, WorthQueryAlwaysReadyTemporalPredicate,
    WorthQueryDesktopClockAdvanceDenial, WorthQueryDesktopClockHandle,
    WorthQueryDesktopClockSource, WorthQueryDesktopTemporalBindingSpec,
    WorthQueryDesktopTemporalHomeBinding, WorthQueryDesktopTemporalInstallationDenial,
    WorthQueryDesktopTemporalNodeSpec, WorthQueryInstalledDesktopTemporal,
    WorthQueryInstalledDesktopTemporalClock, WorthQueryTemporalCatchUpClockStop,
    WorthQueryTemporalCatchUpDenial, WorthQueryTemporalCatchUpPolicy,
    WorthQueryTemporalCatchUpReport, WorthQueryTemporalCatchUpStatus,
    DEFAULT_DESKTOP_TEMPORAL_MAX_MEMORY_BYTES, DEFAULT_DESKTOP_TEMPORAL_MAX_WORK,
    MAX_TEMPORAL_CATCH_UP_STEPS,
};
#[doc(hidden)]
pub use crate::temporal_contribution::{recurring_invocation_failure, recurring_next_occurrence};
pub use crate::temporal_contribution::{
    WorthQueryRecurringClock, WorthQueryRecurringConfig, WorthQueryRecurringInvoker,
    WorthQueryRecurringNode, WorthQueryRecurringOccurrence, WorthQueryRecurringOccurrenceBinding,
    WorthQueryRecurringOwner, WorthQueryRecurringProjector, WorthQueryRecurringRow,
    WorthQueryRecurringRuleIdentity, WorthQueryRecurringScheduleDecision,
    WORTH_QUERY_RECURRING_ACTIVE, WORTH_QUERY_RECURRING_COMPLETED,
    WORTH_QUERY_RECURRING_READY_GATE,
};
