//! Execution and its consumers use the same standard-library capability.
pub(crate) const HAS_OS_THREADS: bool = cfg!(worth_execution_os_threads);

/// Includes items that need OS threads when Execution has that backend.
/// The owner selects this macro with the same capability as its worker pool.
#[cfg(worth_execution_os_threads)]
#[macro_export]
macro_rules! needs_os_threads {
    ($($item:item)*) => { $($item)* };
}

/// Omits items that need OS threads when Execution has no such backend.
#[cfg(not(worth_execution_os_threads))]
#[macro_export]
macro_rules! needs_os_threads {
    ($($item:item)*) => {};
}
