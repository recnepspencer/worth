//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeRetentionLedger>(
            "crates/worth-runtime-bridge/src/conditional_execution/retention/ledger.rs",
            "BridgeRetentionLedger",
            128 * 1024,
        );
        owner.field::<BridgeConditionalRetentionBudget>("budget");
        owner.field::<Mutex<Usage>>("state");
    }
}
