//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalBranchAdvanceOutcome>(
            "crates/worth-signal/src/branch/advancement.rs",
            "SignalBranchAdvanceOutcome",
            128 * 1024,
        );
        owner.field::<AdmittedSignalBranchBasis>("advanced_basis");
        owner.field::<TransactionResult>("transaction");
        owner.field::<Option<super::super::owner_services::SignalConditionalDefinitionAdvanceBinding>>("conditional_definition");
    }
    {
        let mut owner = census.owner::<SignalBranchAdvanceDenial>(
            "crates/worth-signal/src/branch/advancement.rs",
            "SignalBranchAdvanceDenial",
            128 * 1024,
        );
        owner.field::<usize>("OperationCapacityExhausted.maximum_in_flight_operations");
        owner.field::<SignalBranchId>("UnknownBranch.branch_id");
        owner.field::<SignalBranchId>("RetirementInProgress.branch_id");
        owner.field::<SignalBranchId>("RetiredBranch.branch_id");
        owner.field::<SignalBranchId>("QuarantinedBranch.branch_id");
        owner.field::<SignalBranchId>("OwnerCellMisuse.branch_id");
        owner.field::<Vec<FoundationalBranchReferenceMismatchAxis>>("BasisMismatch.axes");
        owner.field::<SignalBranchRetentionAcquisitionDenial>("RetentionUnavailable.denial");
        owner.field::<SignalBranchAdvanceEngineDenial>("MutationDeniedNoMovement.denial");
        owner.field::<SignalError>("MutationFailedNoMovement.error");
    }
}
