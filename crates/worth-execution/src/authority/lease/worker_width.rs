//! The only machine observation used to propose an authority's worker count.
use super::ExecutionAuthorityConfig;
use std::num::NonZeroUsize;

impl ExecutionAuthorityConfig {
    /// Proposes the machine's available workers, or one on a serial platform.
    /// The caller still declares and admits its request's worker budget.
    pub fn available_worker_count() -> NonZeroUsize {
        if crate::backend::HAS_OS_THREADS {
            std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN)
        } else {
            NonZeroUsize::MIN
        }
    }
}
