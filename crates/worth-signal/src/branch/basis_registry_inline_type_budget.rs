//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalBranchBasisRegistry>(
            "crates/worth-signal/src/branch/basis_registry.rs",
            "SignalBranchBasisRegistry",
            128 * 1024,
        );
        owner.field::<Arc<Mutex<SignalBranchBasisRegistryState>>>("state");
    }
}
