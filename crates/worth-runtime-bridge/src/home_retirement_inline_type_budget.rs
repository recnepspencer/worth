//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<BridgeHomeRetirementDenial>(
            "crates/worth-runtime-bridge/src/home_retirement.rs",
            "BridgeHomeRetirementDenial",
            128 * 1024,
        );
    }
    {
        let _owner = census.owner::<BridgeHomeRetirementProgress>(
            "crates/worth-runtime-bridge/src/home_retirement.rs",
            "BridgeHomeRetirementProgress",
            128 * 1024,
        );
    }
}
