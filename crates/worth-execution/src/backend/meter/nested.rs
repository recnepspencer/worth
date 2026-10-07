use worth_foundational::ExecutionReport;

use super::ACTIVE_METER;

/// Add a joined nested computation to the invoking partition, never to a
/// global completion-order counter. The enclosing partition's canonical
/// identity determines whether this cost is ultimately charged.
pub(crate) fn record_nested(report: ExecutionReport, stopped: bool) {
    ACTIVE_METER.with(|active| {
        if let Some(parent) = active.borrow().last() {
            let mut parent = parent.borrow_mut();
            parent.work = match parent.work.checked_add(report.charged_work()) {
                Some(work) => work,
                None => {
                    parent.nested_stopped = true;
                    u64::MAX
                }
            };
            parent.span = match parent.span.checked_add(report.charged_span()) {
                Some(span) => span,
                None => {
                    parent.nested_stopped = true;
                    u64::MAX
                }
            };
            parent.nested_stopped |= stopped;
            if parent.work > parent.limits.ceiling {
                parent.nested_stopped = true;
            }
        }
    });
}
