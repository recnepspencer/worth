//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalAdvanceOutputReservation<'static, (), (), ()>>(
            "crates/worth-signal/src/branch/owner_services/owner/output_retention.rs",
            "SignalAdvanceOutputReservation",
            128 * 1024,
        );
        owner.field::<&'static SignalOwner<(), (), ()>>("owner");
        owner.field::<&'static SignalOwnerOperationAdmission<'static>>("admission");
        owner.field::<Arc<SignalBranchExecutionCell<SignalBranchCellState<(), (), ()>>>>("cell");
        owner.field::<SignalBranchAdmissionReservation>("retention");
    }
}
