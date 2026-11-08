//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<SignalObservationSurface>(
            "crates/worth-signal/src/observation/session/request.rs",
            "SignalObservationSurface",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<SignalObservationRequest>(
            "crates/worth-signal/src/observation/session/request.rs",
            "SignalObservationRequest",
            128 * 1024,
        );
        owner.field::<u8>("mask");
    }
}
