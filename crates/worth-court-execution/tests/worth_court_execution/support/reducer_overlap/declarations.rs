// Rendezvous the first independent reducer calls at their existing kernel door.
pub(crate) use std::time::Duration;
pub(crate) use {std::sync::Condvar, std::sync::Mutex, std::sync::PoisonError};

pub(crate) struct ReducerOverlap {
    pub(crate) arrivals: Mutex<usize>,
    pub(crate) ready: Condvar,
    pub(crate) required: usize,
}

impl ReducerOverlap {
    pub(crate) fn new(required: usize) -> Self {
        Self {
            arrivals: std::sync::Mutex::new(0),
            ready: std::sync::Condvar::new(),
            required,
        }
    }

    pub(crate) fn rendezvous(&self) {
        let mut arrivals = self
            .arrivals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *arrivals += 1;
        if *arrivals >= self.required {
            self.ready.notify_all();
            return;
        }
        let (arrivals, _) = self
            .ready
            .wait_timeout_while(arrivals, std::time::Duration::from_secs(5), |arrivals| {
                *arrivals < self.required
            })
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            *arrivals >= self.required,
            "independent reducer calls never rendezvoused"
        );
    }
}
