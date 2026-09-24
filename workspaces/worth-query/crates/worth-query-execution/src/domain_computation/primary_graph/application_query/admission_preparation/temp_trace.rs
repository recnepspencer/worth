//! Temporary preparation-stage timing probe; remove after House measurement.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const PHASES: usize = 4;

#[derive(Clone, Copy)]
pub(super) enum PreparationPhase {
    Installed = 0,
    Access = 1,
    Controls = 2,
    Parameters = 3,
}

pub(super) struct PreparationProbe {
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

impl PreparationProbe {
    pub(super) fn begin(query: &str) -> Self {
        let enabled =
            *ENABLED.get_or_init(|| std::env::var_os("WORTH_QUERY_ADMISSION_TRACE").is_some());
        Self {
            query: enabled.then(|| query.to_owned()),
            last: enabled.then(Instant::now),
            nanos: [0; PHASES],
        }
    }

    pub(super) fn mark(&mut self, phase: PreparationPhase) {
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
                "WORTH-UI-TEMPORARY-INSTRUMENTATION query-preparation query={query} count={} installed_ns={} access_ns={} controls_ns={} parameters_ns={}",
                total.count,
                total.nanos[0],
                total.nanos[1],
                total.nanos[2],
                total.nanos[3]
            );
        }
    }
}
