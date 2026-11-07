//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeConditionalSignalBasisBinding>(
            "crates/worth-runtime-bridge/src/conditional_execution/signal_basis_binding.rs",
            "BridgeConditionalSignalBasisBinding",
            128 * 1024,
        );
        owner.field::<Arc<BridgeInstalledConditionalLowering>>("lowering");
        owner.field::<super::super::signal_port::BridgeConditionalSignalPort>("signal_port");
    }
}
