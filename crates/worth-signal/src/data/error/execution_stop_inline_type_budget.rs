//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<SignalExecutionFailure>(
            "crates/worth-signal/src/data/error/execution_stop.rs",
            "SignalExecutionFailure",
            128 * 1024,
        );
    }
}
