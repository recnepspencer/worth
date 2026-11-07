//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQuerySharedReadPinRegistry>(
            "workspaces/worth-query/crates/worth-query/src/runtime/shared_read_pins/registry.rs",
            "WorthQuerySharedReadPinRegistry",
            128 * 1024,
        );
        owner.field::<Arc<Mutex<WorthQuerySharedReadPinRegistryState>>>("state");
        owner.field::<Arc<WorthQuerySharedReadCurrentGeneration>>("current_generation");
        owner.field::<WorthQuerySharedReadHotPathMeasurement>("hot_path_measurement");
    }
}
