//! Temporary admission-stage timing probe; remove after the House measurement.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const PHASES: usize = 6;

#[derive(Clone, Copy)]
pub(super) enum AdmissionPhase {
    GraphReview = 0,
    GraphPlan = 1,
    Disclosure = 2,
    Basis = 3,
    Session = 4,
    Authorities = 5,
}

pub(super) struct AdmissionProbe {
    query: Option<String>,
    last: Option<Instant>,
    nanos: [u128; PHASES],
}

struct Totals {
    count: usize,
    nanos: [u128; PHASES],
}

static ENABLED: OnceLock<bool> = OnceLock::new();
static TOTALS: OnceLock<Mutex<BTreeMap<String, Totals>>> = OnceLock::new();

impl AdmissionProbe {
    pub(super) fn begin(query: &str) -> Self {
        let enabled =
            *ENABLED.get_or_init(|| std::env::var_os("WORTH_QUERY_ADMISSION_TRACE").is_some());
        Self {
            query: enabled.then(|| query.to_owned()),
            last: enabled.then(Instant::now),
            nanos: [0; PHASES],
        }
    }

    pub(super) fn mark(&mut self, phase: AdmissionPhase) {
        if let Some(last) = &mut self.last {
            let now = Instant::now();
            self.nanos[phase as usize] = now.duration_since(*last).as_nanos();
            *last = now;
        }
    }

    pub(super) fn record(self) {
        let Some(query) = self.query else { return };
        let mut totals = TOTALS
            .get_or_init(|| Mutex::new(BTreeMap::new()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let total = totals.entry(query.clone()).or_insert(Totals {
            count: 0,
            nanos: [0; PHASES],
        });
        total.count += 1;
        for (sum, current) in total.nanos.iter_mut().zip(self.nanos) {
            *sum += current;
        }
        if total.count % 100 == 0 {
            eprintln!(
                "WORTH-UI-TEMPORARY-INSTRUMENTATION query-admission query={query} count={} graph_review_ns={} graph_plan_ns={} disclosure_ns={} basis_ns={} session_ns={} authorities_ns={}",
                total.count,
                total.nanos[0],
                total.nanos[1],
                total.nanos[2],
                total.nanos[3],
                total.nanos[4],
                total.nanos[5]
            );
        }
    }
}
