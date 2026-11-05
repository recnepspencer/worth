//! Raw root-qualified required-work windows; only the SDK report is accepted cost.
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_cost_stages::{Observer, Stage};
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::cell::RefCell;
use worth_execution::RequestConsultObservationForTest;
struct Trace {
    sdk: RequestConsultObservationForTest,
    events: Vec<(Stage, usize)>,
}
thread_local! { static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) }; }
struct Guard<'a> {
    calls: &'a prefix_observation::Calls,
    previous: Option<Observer>,
}
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.calls.cost_stages.replace(self.previous.take());
        let removed = TRACE.with(|slot| slot.borrow_mut().take());
        drop(removed);
    }
}
fn record(stage: Stage) {
    TRACE.with(|slot| {
        if let Some(trace) = slot.borrow_mut().as_mut() {
            if trace.sdk.is_current_scope_observed_for_test() {
                trace.events.push((stage, trace.sdk.count_for_test()));
            }
        }
    });
}
pub(super) fn invoice<T>(
    calls: &prefix_observation::Calls,
    operation: impl FnOnce() -> T,
) -> (T, u64) {
    let mut guard = None;
    let (result, accepted) = super::invoice_observing(
        |context| {
            TRACE.with(|slot| {
                assert!(slot.borrow().is_none());
                *slot.borrow_mut() = Some(Trace {
                    sdk:
                        RequestConsultObservationForTest::install_in_scope_on_this_thread_for_test(
                            context,
                            |_| {},
                        )
                        .unwrap(),
                    events: Vec::new(),
                });
            });
            guard = Some(Guard {
                calls,
                previous: calls.cost_stages.replace(Some(Arc::new(record))),
            });
        },
        operation,
    );
    TRACE.with(|slot| {
        let slot = slot.borrow();
        let trace = slot.as_ref().unwrap();
        let prefixes = trace.sdk.charged_prefixes_for_test();
        if prefixes.last().map(|(_, work)| *work) != Some(accepted) {
            eprintln!("raw required-prefix tail differs from accepted report; no phase-cost equality inferred");
        }
        report(&trace.events, &prefixes, accepted);
    });
    drop(guard);
    (result, accepted)
}
fn bounds(prefixes: &[(usize, u64)], consult: usize) -> (u64, u64, usize, usize) {
    let begin = prefixes.partition_point(|(number, _)| *number < consult);
    let end = prefixes.partition_point(|(number, _)| *number <= consult);
    let prior = begin.checked_sub(1).map_or(0, |n| prefixes[n].1);
    let (mut lower, mut upper) = (prior, prior);
    for (_, work) in &prefixes[begin..end] {
        lower = lower.min(*work);
        upper = upper.max(*work);
    }
    (lower, upper, begin.saturating_sub(1), end)
}
fn report(events: &[(Stage, usize)], prefixes: &[(usize, u64)], accepted: u64) {
    if prefixes.is_empty() {
        eprintln!("no raw SDK prefixes; all callback windows unqualified, accepted={accepted}");
        return;
    }
    let mut stack = Vec::new();
    let mut totals = [(0usize, 0u64, 0u64, 0usize); 4];
    for &(stage, consult) in events {
        let (kind, begin) = match stage {
            Stage::UpdateBegin => (0, true),
            Stage::UpdateEnd => (0, false),
            Stage::MergeBegin => (1, true),
            Stage::MergeEnd => (1, false),
            Stage::BoundsBegin => (2, true),
            Stage::BoundsEnd => (2, false),
            Stage::LoanBegin => (3, true),
            Stage::LoanEnd => (3, false),
        };
        if begin {
            stack.push((kind, consult));
            continue;
        }
        let Some(position) = stack.iter().rposition(|(opened, _)| *opened == kind) else {
            totals[kind].3 += 1;
            continue;
        };
        for (unfinished, _) in stack.drain(position + 1..) {
            totals[unfinished].3 += 1;
        }
        let (_, first) = stack.pop().unwrap();
        let a = bounds(prefixes, first);
        let b = bounds(prefixes, consult);
        let total = &mut totals[kind];
        total.0 += 1;
        if prefixes[a.2..b.3].windows(2).all(|p| p[0].1 <= p[1].1) {
            total.1 += b.0.saturating_sub(a.1);
            total.2 += b.1.saturating_sub(a.0);
        } else {
            total.3 += 1;
        }
    }
    for (unfinished, _) in stack {
        totals[unfinished].3 += 1;
    }
    eprintln!("prefix SDK accepted whole invoice={accepted}; overlapping raw REQUIRED windows, not exclusive accepted phases:");
    for (label, total) in ["update", "merge", "bounds", "row loan"]
        .into_iter()
        .zip(totals)
    {
        eprintln!("  {label}: occurrences={} monotone raw-difference-sum=({},{}) unqualified-nonmonotone={}",total.0,total.1,total.2,total.3);
    }
    eprintln!(
        "  raw required-prefix decreases across entire operation={}",
        prefixes.windows(2).filter(|p| p[0].1 > p[1].1).count()
    );
}
