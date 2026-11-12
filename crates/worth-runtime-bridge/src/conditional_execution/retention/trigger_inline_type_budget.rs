//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeRetainedTrigger>(
            "crates/worth-runtime-bridge/src/conditional_execution/retention/trigger.rs",
            "BridgeRetainedTrigger",
            128 * 1024,
        );
        owner.field::<BridgeDeliveredCorrespondenceChangeSet>("change_set");
        owner.field::<BridgeRetentionReservation>("_reservation");
    }
}
