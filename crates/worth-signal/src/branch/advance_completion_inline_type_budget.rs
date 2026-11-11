//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalBranchAdvanceCompletion>(
            "crates/worth-signal/src/branch/advance_completion.rs",
            "SignalBranchAdvanceCompletion",
            128 * 1024,
        );
        owner.field::<Option<Result<SignalBranchAdvanceOutcome, SignalBranchAdvanceDenial>>>(
            "result",
        );
        owner.field::<Option<Box<dyn Any + Send>>>("unwind");
    }
}
