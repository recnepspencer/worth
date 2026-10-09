//! Search only setup and admission steps; reached-subject assertions remain fatal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::checkpoint_recovery) enum Attempt {
    Below(&'static str),
    Hit,
    Above(&'static str),
}

#[derive(Clone, Copy)]
pub(in crate::checkpoint_recovery) enum Goal {
    Hit,
    LowerEdge,
    UpperEdge,
}

#[derive(Debug)]
pub(in crate::checkpoint_recovery) struct Search {
    pub edge: Option<usize>,
    pub probes: Vec<(usize, Attempt)>,
}
impl Search {
    pub fn require_hit(&self, name: &str) -> usize {
        self.edge
            .unwrap_or_else(|| panic!("{name}: no hit; probes={:?}", self.probes))
    }
}

/// At most twenty deterministic runs find a hit or its requested band edge.
/// A workload returns Below only before its subject, and Above only when the
/// refused or reclaimed work instead fits. Assertions within the subject panic.
pub(in crate::checkpoint_recovery) fn search(
    name: &str,
    mut floor: usize,
    mut ample: usize,
    goal: Goal,
    mut workload: impl FnMut(usize) -> Attempt,
) -> Search {
    assert!(
        floor > 0 && floor <= ample,
        "{name}: search interval is positive and nonempty"
    );
    let mut result = Search {
        edge: None,
        probes: Vec::new(),
    };
    for _ in 0..20 {
        if floor > ample {
            break;
        }
        let knob = floor + (ample - floor) / 2;
        // A reached-subject panic stays fatal; its context names the attempted
        // knob and earlier answers without treating the assertion as a search answer.
        let answer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| workload(knob)))
            .unwrap_or_else(|failure| {
                let detail = failure.downcast_ref::<String>().map(String::as_str)
                    .or_else(|| failure.downcast_ref::<&str>().copied())
                    .unwrap_or("non-string assertion failure");
                panic!("{name}: subject assertion failed at run {}, knob {knob}; earlier={:?}; {detail}",
                    result.probes.len() + 1, result.probes);
            });
        result.probes.push((knob, answer));
        match answer {
            Attempt::Below(_) => floor = knob + 1,
            Attempt::Hit => {
                result.edge = Some(knob);
                if matches!(goal, Goal::Hit) {
                    break;
                }
                match goal {
                    Goal::LowerEdge => ample = knob - 1,
                    Goal::UpperEdge => floor = knob + 1,
                    Goal::Hit => unreachable!(),
                }
            }
            Attempt::Above(_) => ample = knob - 1,
        }
    }
    result
}

macro_rules! start {
    ($result:expr, $step:literal) => {
        match $result {
            Ok(value) => value,
            Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded =>
                return support::capacity_region::Attempt::Below($step),
            Err(other) => panic!("{}: unexpected setup refusal: {other:?}", $step),
        }
    };
}
pub(in crate::checkpoint_recovery) use start;

macro_rules! settle {
    ($demand:expr, $request:expr, $step:literal) => {{
        let mut settled = None;
        for _ in 0..256 {
            match $demand.advance(&$request) {
                Ok(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandProgress::Pending) => (),
                Ok(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandProgress::Settled(value)) => { settled = Some(value); break; },
                Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                    if denial.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded =>
                    return support::capacity_region::Attempt::Below($step),
                Err(other) => panic!("{}: unexpected setup refusal: {other:?}", $step),
            }
        }
        settled.expect(concat!($step, " settles within 256 advances"))
    }};
}
pub(in crate::checkpoint_recovery) use settle;
