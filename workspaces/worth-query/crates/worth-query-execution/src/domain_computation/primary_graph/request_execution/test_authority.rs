//! The one process authority shared by Query test callers and placements.

use std::{num::NonZeroUsize, sync::OnceLock};
use worth_execution::{ExecutionAuthority, ExecutionAuthorityConfig};

/// The most workers a test lease may ask for: twice the machine's width.
pub fn test_execution_workers_for_test() -> NonZeroUsize {
    std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .saturating_mul(NonZeroUsize::new(2).expect("two is not zero"))
}

/// This process's one authority, admitting [`test_execution_workers_for_test`].
pub fn query_test_execution_authority_for_test() -> &'static ExecutionAuthority {
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: test_execution_workers_for_test(),
            charged_memory_bytes: None,
        })
        .expect("a test process constructs one authority, here")
    })
}
