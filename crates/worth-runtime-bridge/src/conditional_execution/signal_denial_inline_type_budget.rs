//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<BridgeSignalDenial>(
            "crates/worth-runtime-bridge/src/conditional_execution/signal_denial.rs",
            "BridgeSignalDenial",
            128 * 1024,
        );
    }
}
