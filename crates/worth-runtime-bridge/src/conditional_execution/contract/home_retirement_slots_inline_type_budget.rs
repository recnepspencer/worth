//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<OrdinaryBridgeCustody>("crates/worth-runtime-bridge/src/conditional_execution/contract/home_retirement_slots.rs", "OrdinaryBridgeCustody", 128 * 1024);
        owner.field::<Option<RuntimeBridge>>("runtime");
    }
    {
        let mut owner = census.owner::<SignalRuntimeGuard<'static>>("crates/worth-runtime-bridge/src/conditional_execution/contract/home_retirement_slots.rs", "SignalRuntimeGuard", 128 * 1024);
        owner.field::<MutexGuard<'static, Option<Signal>>>("guard");
    }
}
