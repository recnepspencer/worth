//! Owner-recorded advancement cost and semantic application order.
use super::WorthQueryAdvancementDenial;
use std::cell::RefCell;
use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

/// An effect committed by the owner, or a product/stable-output publication it completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAdvancementEventKind {
    Commit,
    Publication,
}

/// Semantic identity recorded by the owner that performed an application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryAdvancementIdentity {
    /// A required producer occurrence. The source commitment includes its query,
    /// parameters and observed facts, and excludes runtime allocation ordinals.
    Producer {
        producer: String,
        query: [u8; 32],
        parameters: [u8; 32],
        root: worth_relational::facade::identity::EntityId,
        source: [u8; 32],
    },
    /// An ordinary mutation's installed operation and canonical idempotency input.
    Mutation {
        operation: [u8; 32],
        key: [u8; 32],
        intent: [u8; 32],
    },
}
/// An owner-applied event, never a worker completion notification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdvancementEvent {
    pub kind: WorthQueryAdvancementEventKind,
    pub identity: WorthQueryAdvancementIdentity,
}

/// One attempted advancement, including refusal and the actually applied prefix.
/// Semantic caller failures retain the report of their admitted advancement.
/// A refusal before admission reports zero work/span and no application events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdvancementObservation {
    pub report: ExecutionReport,
    pub refusal: Option<WorthQueryAdvancementDenial>,
    pub applied: Vec<WorthQueryAdvancementEvent>,
}

struct ActiveObservation {
    posture: ExecutionPosture,
    applied: Vec<WorthQueryAdvancementEvent>,
}
thread_local! {
    static REQUESTS: RefCell<Vec<WorthQueryAdvancementObservation>> = const { RefCell::new(Vec::new()) };
    static ACTIVE: RefCell<Option<ActiveObservation>> = const { RefCell::new(None) };
}
pub(super) fn begin(posture: ExecutionPosture) {
    ACTIVE.with(|active| {
        let previous = active.replace(Some(ActiveObservation {
            posture,
            applied: Vec::new(),
        }));
        assert!(previous.is_none(), "one observation per active advancement");
    });
}
pub(super) fn record_report(report: ExecutionReport) {
    finish(report, None);
}
fn finish(report: ExecutionReport, refusal: Option<WorthQueryAdvancementDenial>) {
    let active = ACTIVE
        .with(|active| active.borrow_mut().take())
        .expect("the advancement owns its observation");
    REQUESTS.with(|requests| {
        requests
            .borrow_mut()
            .push(WorthQueryAdvancementObservation {
                report,
                refusal,
                applied: active.applied,
            })
    });
}
pub(super) fn record_result<R>(result: &Result<R, WorthQueryAdvancementDenial>) {
    if let Err(cause) = result {
        let posture = ACTIVE.with(|active| {
            active
                .borrow()
                .as_ref()
                .expect("the refused advancement is active")
                .posture
        });
        finish(
            ExecutionReport::new(posture, 0, 0, ExecutionPhysicalReport::default()),
            Some(*cause),
        );
    }
}

/// Drains the existing observer's queue with cost and owner application evidence.
/// This and the report-only getter consume the same records.
pub fn advancement_observations_on_this_thread_for_test() -> Vec<WorthQueryAdvancementObservation> {
    REQUESTS.with(|requests| std::mem::take(&mut *requests.borrow_mut()))
}
/// Takes this thread's completed request reports, including admission refusals.
pub fn advancement_requests_on_this_thread_for_test(
) -> Vec<Result<ExecutionReport, WorthQueryAdvancementDenial>> {
    advancement_observations_on_this_thread_for_test()
        .into_iter()
        .map(|observation| match observation.refusal {
            Some(cause) => Err(cause),
            None => Ok(observation.report),
        })
        .collect()
}

pub(in crate::domain_computation::primary_graph) fn record_event(
    event: WorthQueryAdvancementEvent,
) {
    ACTIVE.with(|active| {
        if let Some(active) = active.borrow_mut().as_mut() {
            active.applied.push(event);
        }
    });
}

#[cfg(feature = "test-query-execution-observer")]
thread_local! {
    static CALLER_PASSES: RefCell<Vec<(Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>, u64)>> = const { RefCell::new(Vec::new()) };
}
/// Observes pass results without adding an execution scope or changing control flow.
#[cfg(feature = "test-query-execution-observer")]
pub fn caller_pass_reports_on_this_thread_for_test() -> Vec<(
    Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>,
    u64,
)> {
    CALLER_PASSES.with(|passes| std::mem::take(&mut *passes.borrow_mut()))
}
#[cfg(feature = "test-query-execution-observer")]
pub(in crate::domain_computation::primary_graph) fn record_caller_pass<R>(
    result: &Result<R, crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial>,
) {
    CALLER_PASSES.with(|passes| {
        passes.borrow_mut().push((
            result.as_ref().map(|_| ()).map_err(|denial| denial.kind()),
            worth_signal::facade::observed_signal_request_work_on_this_thread_for_test(),
        ))
    });
}

pub(super) fn record_panicked(report: Option<ExecutionReport>) {
    if let Some(report) = report {
        finish(report, Some(WorthQueryAdvancementDenial::Panicked));
    } else {
        record_result::<()>(&Err(WorthQueryAdvancementDenial::Panicked));
    }
}

pub(super) fn record_opening_refusal(cause: WorthQueryAdvancementDenial) {
    let posture = ACTIVE.with(|active| {
        active
            .borrow()
            .as_ref()
            .map_or(ExecutionPosture::Serial, |active| active.posture)
    });
    REQUESTS.with(|requests| {
        requests
            .borrow_mut()
            .push(WorthQueryAdvancementObservation {
                report: ExecutionReport::new(posture, 0, 0, ExecutionPhysicalReport::default()),
                refusal: Some(cause),
                applied: Vec::new(),
            })
    });
}

pub(in crate::domain_computation::primary_graph) mod application;
