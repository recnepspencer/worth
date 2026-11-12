//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeObservationBaselines>("crates/worth-runtime-bridge/src/conditional_execution/observation_retention/baseline.rs", "BridgeObservationBaselines", 128 * 1024);
        owner.field::<Arc<BridgeRetentionLedger>>("ledger");
        owner.field::<Mutex<BridgeRetainedObservations>>("current");
        owner.field::<BridgeRetentionReservation>("_reservation");
    }
}
