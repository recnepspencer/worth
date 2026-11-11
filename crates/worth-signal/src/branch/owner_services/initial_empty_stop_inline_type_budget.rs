//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<SignalInitialEmptyStopDenial>(
            "crates/worth-signal/src/branch/owner_services/initial_empty_stop.rs",
            "SignalInitialEmptyStopDenial",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<SignalInitialEmptyStopRefused<(), (), (), (), ()>>(
            "crates/worth-signal/src/branch/owner_services/initial_empty_stop.rs",
            "SignalInitialEmptyStopRefused",
            128 * 1024,
        );
        owner.field::<SignalRuntime<(), (), (), (), ()>>("runtime");
        owner.field::<SignalInitialEmptyStopDenial>("reason");
    }
    {
        let mut owner = census.owner::<PreparedInitialEmptySignalStop<(), (), (), (), ()>>(
            "crates/worth-signal/src/branch/owner_services/initial_empty_stop.rs",
            "PreparedInitialEmptySignalStop",
            128 * 1024,
        );
        owner.field::<SignalRuntime<(), (), (), (), ()>>("runtime");
    }
    {
        let mut owner = census.owner::<StoppedInitialEmptySignalAllocation<(), (), (), (), ()>>(
            "crates/worth-signal/src/branch/owner_services/initial_empty_stop.rs",
            "StoppedInitialEmptySignalAllocation",
            128 * 1024,
        );
        owner.field::<SignalRuntime<(), (), (), (), ()>>("_runtime");
    }
}
