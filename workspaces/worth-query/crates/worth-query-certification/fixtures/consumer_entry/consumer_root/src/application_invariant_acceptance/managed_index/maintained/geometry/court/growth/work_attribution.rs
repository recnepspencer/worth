//! Callback anchors in the actual original SDK invoice; gaps retain ambiguity.
use super::*;
use provider::cost_stages::{Observer, Stage};
use std::cell::RefCell;
use worth_execution::RequestConsultObservationForTest;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Audience {
    RuleIndex,
    PublicIndex,
}
const AUDIENCES: [Audience; 2] = [Audience::RuleIndex, Audience::PublicIndex];
struct Trace {
    sdk: RequestConsultObservationForTest,
    events: Vec<(Audience, Stage, usize)>,
}
thread_local! { static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) }; }
struct Guard<'a> {
    calls: [&'a provider::Calls; 2],
    previous: [Option<Observer>; 2],
}
impl<'a> Guard<'a> {
    fn arm(
        calls: [&'a provider::Calls; 2],
        context: &worth_execution::MapKernelContext<'_, '_>,
    ) -> Self {
        TRACE.with(|slot| {
            let mut slot = slot.borrow_mut();
            assert!(slot.is_none(), "one issuing-thread invoice observation");
            *slot = Some(Trace {
                sdk: RequestConsultObservationForTest::install_in_scope_on_this_thread_for_test(
                    context,
                    |_| {},
                )
                .unwrap(),
                events: Vec::new(),
            });
        });
        Self {
            calls,
            previous: std::array::from_fn(|index| {
                let audience = AUDIENCES[index];
                calls[index]
                    .cost_stages
                    .replace(Some(Arc::new(move |stage| record(audience, stage))))
            }),
        }
    }
}
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        for (calls, previous) in self.calls.iter().zip(self.previous.iter_mut()) {
            calls.cost_stages.replace(previous.take());
        }
        // Drop the SDK guard outside our own RefCell borrow, including unwind.
        let removed = TRACE.with(|slot| slot.borrow_mut().take());
        drop(removed);
    }
}
fn record(audience: Audience, stage: Stage) {
    TRACE.with(|slot| {
        if let Some(trace) = slot.borrow_mut().as_mut() {
            if !trace.sdk.is_current_scope_observed_for_test() {
                return;
            }
            let consult = trace.sdk.count_for_test();
            trace.events.push((audience, stage, consult));
        }
    });
}
pub(super) fn invoice<T>(calls: [&provider::Calls; 2], operation: impl FnOnce() -> T) -> (T, u64) {
    let before = calls.map(super::public_work::Snapshot::capture);
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let mut guard = None;
    let (result, report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&original), |context| {
            guard = Some(Guard::arm(calls, context));
            operation()
        })
        .unwrap();
    let invoice = report.charged_work();
    TRACE.with(|slot| {
        let slot = slot.borrow();
        let trace = slot.as_ref().unwrap();
        let prefixes = trace.sdk.charged_prefixes_for_test();
        // Required prefixes on rejected SDK work are not accepted invoice work.
        // These actual success/domain-refusal journeys must close exactly.
        assert_eq!(prefixes.last().map(|(_, work)| *work), Some(invoice));
        assert!(prefixes.windows(2).all(|p| p[0].0 <= p[1].0));
        let last_anchor = trace.events.last().expect("actual provider callbacks").2;
        let callback_end = prefixes.partition_point(|(consult, _)| *consult <= last_anchor);
        assert!(
            prefixes[..callback_end]
                .windows(2)
                .all(|p| p[0].1 <= p[1].1),
            "raw required-work windows refuse an unsettled callback interval"
        );
        for pair in prefixes[callback_end.saturating_sub(1)..].windows(2) {
            if pair[0].1 > pair[1].1 {
                eprintln!("  uninterpreted settlement-tail required-prefix decrease: {pair:?}");
            }
        }
        assert!(trace.events.windows(2).all(|p| p[0].2 <= p[1].2));
        for audience in AUDIENCES {
            for stage in [Stage::UpdateBegin, Stage::UpdateEnd] {
                assert_eq!(
                    trace
                        .events
                        .iter()
                        .filter(|(a, s, _)| *a == audience && *s == stage)
                        .count(),
                    2,
                    "both actual scopes have a complete {audience:?} update"
                );
            }
        }
        let bounded = bound_events(&trace.events, &prefixes);
        report_intervals(&bounded, invoice);
    });
    drop(guard);
    for index in 0..2 {
        eprintln!(
            "actual {:?} provider delta {:?}",
            AUDIENCES[index],
            before[index].delta(calls[index])
        );
    }
    (result, invoice)
}

#[derive(Clone, Copy)]
struct Bounds {
    lower: u64,
    upper: u64,
}
fn bound_events(
    events: &[(Audience, Stage, usize)],
    prefixes: &[(usize, u64)],
) -> Vec<(Audience, Stage, Bounds)> {
    let (mut cursor, mut current, mut previous) = (0, 0, None);
    let mut bounds = Bounds { lower: 0, upper: 0 };
    events
        .iter()
        .map(|&(audience, stage, consult)| {
            if previous != Some(consult) {
                while cursor < prefixes.len() && prefixes[cursor].0 < consult {
                    current = prefixes[cursor].1;
                    cursor += 1;
                }
                let lower = current;
                while cursor < prefixes.len() && prefixes[cursor].0 == consult {
                    current = prefixes[cursor].1;
                    cursor += 1;
                }
                bounds = Bounds {
                    lower,
                    upper: current,
                };
                previous = Some(consult);
            }
            (audience, stage, bounds)
        })
        .collect()
}
fn report_intervals(events: &[(Audience, Stage, Bounds)], invoice: u64) {
    let mut previous = Bounds { lower: 0, upper: 0 };
    let mut start = 0;
    let mut summary_groups = [0; 2];
    eprintln!(
        "root-qualified active-stack required-work intervals; separate accepted SDK invoice={invoice}; gaps include framework work"
    );
    while start < events.len() {
        let (audience, stage, _) = events[start];
        let mut end = start + 1;
        while end < events.len() && events[end].0 == audience && events[end].1 == stage {
            end += 1;
        }
        let first = events[start].2;
        let last = events[end - 1].2;
        let gap = (
            first.lower.saturating_sub(previous.upper),
            first.upper.saturating_sub(previous.lower),
        );
        let within = (
            last.lower.saturating_sub(first.upper),
            last.upper.saturating_sub(first.lower),
        );
        eprintln!(
            "  {audience:?} {stage:?} callbacks={} preceding unattributed gap={gap:?}, first-to-last={within:?}",
            end - start
        );
        summary_groups[usize::from(audience == Audience::PublicIndex)] +=
            usize::from(stage == Stage::Summary);
        previous = last;
        start = end;
    }
    assert_eq!(
        summary_groups,
        [2, 2],
        "actual summary qualification for both scopes of both providers"
    );
    eprintln!(
        "  unattributed settlement tail: last required anchor={:?}, accepted invoice={invoice}",
        (previous.lower, previous.upper)
    );
}
