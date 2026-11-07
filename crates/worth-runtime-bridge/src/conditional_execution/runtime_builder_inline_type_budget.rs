//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeConditionalRuntimeBuilder>(
            "crates/worth-runtime-bridge/src/conditional_execution/runtime_builder.rs",
            "BridgeConditionalRuntimeBuilder",
            128 * 1024,
        );
        owner.field::<BridgeOwnedSignalRuntime>("runtime");
        owner.field::<std::collections::BTreeMap<
            Arc<str>,
            super::super::managed_time::BridgeManagedClockDeclaration,
        >>("managed_clocks");
    }
}
