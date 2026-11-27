//! A held trusted clock the court can make unavailable after the Home opens,
//! so a refusal is attributable to the operation, not to Home installation.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::primary_graph::{
    WorthQueryFixedRuntimeTimeSource, WorthQueryRuntimeTimeSource,
    WorthQueryRuntimeTimeSourceDenial,
};

#[derive(Clone)]
pub struct Switchable {
    pub held: WorthQueryFixedRuntimeTimeSource,
    pub available: Arc<AtomicBool>,
}

impl Switchable {
    pub fn new(held: WorthQueryFixedRuntimeTimeSource) -> Self {
        Self {
            held,
            available: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn fail(&self) {
        self.available.store(false, Ordering::SeqCst);
    }
}

impl WorthQueryRuntimeTimeSource for Switchable {
    fn admission_clock(&self) -> WorthQueryAdmissionClock {
        self.held.admission_clock()
    }

    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        if self.available.load(Ordering::SeqCst) {
            self.held.current_time()
        } else {
            Err(WorthQueryRuntimeTimeSourceDenial::Unavailable)
        }
    }
}
