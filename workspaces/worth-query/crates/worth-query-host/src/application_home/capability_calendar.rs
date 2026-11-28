//! The Home's capability calendar travels with its trusted time source, so
//! every opening path that receives the source receives the calendar too.
use std::sync::Arc;

use worth_query_execution::facade::primary_graph::{
    WorthQueryCapabilityCalendar, WorthQueryRuntimeClock,
};

/// `source` (the system clock when absent) carrying `calendar`.
pub(super) fn with_calendar(
    source: Option<WorthQueryRuntimeClock>,
    calendar: Arc<dyn WorthQueryCapabilityCalendar>,
) -> WorthQueryRuntimeClock {
    source
        .unwrap_or_else(WorthQueryRuntimeClock::system)
        .with_capability_calendar(calendar)
}

/// `source`, keeping any calendar the replaced source carried.
pub(super) fn replacing_source(
    replaced: Option<&WorthQueryRuntimeClock>,
    source: WorthQueryRuntimeClock,
) -> WorthQueryRuntimeClock {
    match replaced.and_then(|previous| previous.capability_calendar()) {
        Some(calendar) => source.with_capability_calendar(Arc::clone(calendar)),
        None => source,
    }
}
