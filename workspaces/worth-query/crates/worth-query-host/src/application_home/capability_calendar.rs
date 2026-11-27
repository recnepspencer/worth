//! The Home's capability calendar travels with its trusted time source, so
//! every opening path that receives the source receives the calendar too.
use std::sync::Arc;

use worth_query_execution::facade::primary_graph::{
    WorthQueryCalendaredRuntimeTimeSource, WorthQueryCapabilityCalendar,
    WorthQueryRuntimeTimeSource, WorthQuerySystemRuntimeTimeSource,
};

pub(super) type BoxedTimeSource = Box<dyn WorthQueryRuntimeTimeSource>;

/// `source` (the system clock when absent) carrying `calendar`.
pub(super) fn with_calendar(
    source: Option<BoxedTimeSource>,
    calendar: Arc<dyn WorthQueryCapabilityCalendar>,
) -> BoxedTimeSource {
    let source = source.unwrap_or_else(|| Box::new(WorthQuerySystemRuntimeTimeSource));
    Box::new(WorthQueryCalendaredRuntimeTimeSource::new(source, calendar))
}

/// `source`, keeping any calendar the replaced source carried.
pub(super) fn replacing_source(
    replaced: Option<&BoxedTimeSource>,
    source: BoxedTimeSource,
) -> BoxedTimeSource {
    match replaced.and_then(|previous| previous.capability_calendar()) {
        Some(calendar) => with_calendar(Some(source), calendar),
        None => source,
    }
}
