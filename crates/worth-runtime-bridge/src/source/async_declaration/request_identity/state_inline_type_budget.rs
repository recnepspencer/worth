//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<RuntimeThreadBinding>(
            "crates/worth-runtime-bridge/src/source/async_declaration/request_identity/state.rs",
            "RuntimeThreadBinding",
            128 * 1024,
        );
        owner.field::<ThreadId>("thread");
        owner.field::<Arc<Mutex<Vec<u64>>>>("retired");
    }
    {
        let mut owner = census.owner::<SignalRuntimeThreadAffinityError>(
            "crates/worth-runtime-bridge/src/source/async_declaration/request_identity/state.rs",
            "SignalRuntimeThreadAffinityError",
            128 * 1024,
        );
        owner.field::<u64>("runtime_key");
        owner.field::<ThreadId>("owner");
        owner.field::<ThreadId>("current");
    }
}
