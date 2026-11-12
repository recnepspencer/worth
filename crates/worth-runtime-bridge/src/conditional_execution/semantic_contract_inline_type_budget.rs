//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<BridgeConditionalCondition>(
            "crates/worth-runtime-bridge/src/conditional_execution/semantic_contract.rs",
            "BridgeConditionalCondition",
            128 * 1024,
        );
    }
}
