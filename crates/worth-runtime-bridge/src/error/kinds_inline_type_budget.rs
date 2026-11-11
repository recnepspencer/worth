//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<BridgeDeliveryErrorKind>(
            "crates/worth-runtime-bridge/src/error/kinds.rs",
            "BridgeDeliveryErrorKind",
            128 * 1024,
        );
    }
}
