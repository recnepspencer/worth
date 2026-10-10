//! The one process authority shared by Query test callers and placements.

use std::{num::NonZeroUsize, sync::OnceLock};
use worth_execution::{ExecutionAuthority, ExecutionAuthorityConfig};

/// The most workers a test lease may ask for: twice the machine's width.
pub fn test_execution_workers() -> NonZeroUsize {
    std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .saturating_mul(NonZeroUsize::new(2).expect("two is not zero"))
}

/// This process's one authority, admitting [`test_execution_workers`].
pub fn test_execution_authority() -> &'static ExecutionAuthority {
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: test_execution_workers(),
            charged_memory_bytes: Some(1 << 40),
        })
        .expect("a test process constructs one authority, here")
    })
}
