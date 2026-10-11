//! Rendezvous the first independent reducer calls at their existing kernel door.
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::Duration;

pub(crate) struct ReducerOverlap {
    arrivals: Mutex<usize>,
    ready: Condvar,
    required: usize,
}

impl ReducerOverlap {
    pub(crate) fn new(required: usize) -> Self {
        Self {
            arrivals: Mutex::new(0),
            ready: Condvar::new(),
            required,
        }
    }

    pub(crate) fn rendezvous(&self) {
        let mut arrivals = self.arrivals.lock().unwrap_or_else(PoisonError::into_inner);
        *arrivals += 1;
        if *arrivals >= self.required {
            self.ready.notify_all();
            return;
        }
        let (arrivals, _) = self
            .ready
            .wait_timeout_while(arrivals, Duration::from_secs(5), |arrivals| {
                *arrivals < self.required
            })
            .unwrap_or_else(PoisonError::into_inner);
        assert!(
            *arrivals >= self.required,
            "independent reducer calls never rendezvoused"
        );
    }
}
