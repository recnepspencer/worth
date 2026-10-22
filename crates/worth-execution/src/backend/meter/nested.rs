use worth_foundational::ExecutionReport;

use super::{NestedStopOwner, ACTIVE_METER};

/// Uncharged oracle/frontier lanes defer cost, never an established request stop.
pub(crate) fn record_nested_outcome(
    report: ExecutionReport,
    stop: Option<NestedStopOwner>,
    charge_parent: bool,
) {
    if charge_parent {
        record_nested(report, stop);
    } else if stop == Some(NestedStopOwner::Request) {
        super::request_stop(());
    }
}

/// Add a joined nested computation to the invoking partition, never to a
/// global completion-order counter. The enclosing partition's canonical
/// identity determines whether this cost is ultimately charged.
pub(crate) fn record_nested(report: ExecutionReport, stop: Option<NestedStopOwner>) {
    ACTIVE_METER.with(|active| {
        if let Some(parent) = active.borrow().last() {
            let mut parent = parent.borrow_mut();
            if let Some(capture) = parent.capture_mut() {
                capture.invalidate();
            }
            let mut overflowed = false;
            if let (Some(accounting), Some((work, span))) =
                (parent.accounting.as_mut(), report.accounted_cost())
            {
                accounting.work = accounting.work.checked_add(work).unwrap_or_else(|| {
                    overflowed = true;
                    u64::MAX
                });
                accounting.span = accounting.span.checked_add(span).unwrap_or_else(|| {
                    overflowed = true;
                    u64::MAX
                });
            }
            if overflowed {
                parent.nested_stop.record(NestedStopOwner::Request);
            }
            if let Some(owner) = stop {
                parent.nested_stop.record(owner);
            }
            if parent
                .limits
                .ceiling
                .is_some_and(|ceiling| parent.work() > ceiling)
            {
                let owner = parent.limits.ceiling_owner;
                parent.nested_stop.record(owner);
            }
        }
    });
}
