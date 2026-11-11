//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalBranchAdvancePreflight>(
            "crates/worth-signal/src/logic/transaction/runtime/state/branching/advancement.rs",
            "SignalBranchAdvancePreflight",
            128 * 1024,
        );
        owner.field::<SignalBranchHandle>("branch");
        owner.field::<SignalBranchAdmissionLease>("retention");
        owner.field::<SignalBranchAdvanceLane>("lane");
    }
    {
        let _owner = census.owner::<SignalBranchAdvanceLane>(
            "crates/worth-signal/src/logic/transaction/runtime/state/branching/advancement.rs",
            "SignalBranchAdvanceLane",
            128 * 1024,
        );
    }
}
