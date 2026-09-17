use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use worth_query_host::facade::primary_graph::{
    WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial,
};

pub(super) struct TrackedAuthorizationTime {
    releases: Arc<AtomicUsize>,
}

impl TrackedAuthorizationTime {
    pub(super) fn new(releases: Arc<AtomicUsize>) -> Self {
        Self { releases }
    }
}

impl WorthQueryRuntimeTimeSource for TrackedAuthorizationTime {
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        Ok(SystemTime::now())
    }
}

impl Drop for TrackedAuthorizationTime {
    fn drop(&mut self) {
        self.releases.fetch_add(1, Ordering::SeqCst);
    }
}
