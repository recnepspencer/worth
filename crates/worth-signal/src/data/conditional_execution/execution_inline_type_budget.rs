//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalConditionalExecutionRequest<'static>>(
            "crates/worth-signal/src/data/conditional_execution/execution.rs",
            "SignalConditionalExecutionRequest",
            128 * 1024,
        );
        owner.field::<&'static InstalledSignalConditionalContract>("contract");
        owner.field::<&'static str>("snapshot_identity");
        owner.field::<&'static str>("execution_identity");
        owner.field::<u64>("attempt");
        owner.field::<bool>("force_on_demand");
    }
}
