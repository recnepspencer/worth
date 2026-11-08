//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<SignalConditionalVersionComparator>(
            "crates/worth-signal/src/data/conditional_execution/contract.rs",
            "SignalConditionalVersionComparator",
            128 * 1024,
        );
    }
}
