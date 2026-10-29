pub(crate) use {std::ptr};

pub(crate) use {worth_foundational::FoundationalBranchReferenceMismatchAxis, worth_foundational::FoundationalBranchTarget};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::branch::SignalBranchAdvanceDenial, worth_signal::facade::branch::SignalBranchBasisCompatibilityDenial, worth_signal::facade::branch::SignalBranchBasisReadmissionDenial, worth_signal::facade::branch::SignalBranchForkOperationDenial, worth_signal::facade::branch::SignalBranchRetentionReleaseDenial, worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome, worth_signal::facade::branch::SignalBranchTarget, worth_signal::facade::branch::SignalOwnerServiceCostSnapshot, worth_signal::facade::branch::SignalOwnerUnavailable, worth_signal::facade::branch::SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION};
pub(crate) use {worth_signal::facade::runtime::SignalBranchRetirementDenial};
pub(crate) use {worth_signal::facade::SignalBranchId, worth_signal::facade::SignalBranchRetirementReason, worth_signal::facade::SignalError, worth_signal::facade::SignalGraph, worth_signal::facade::SignalRuntime, worth_signal::facade::TransactionOutcome};

pub(crate) fn runtime() -> SignalRuntime<(), (), (), (), ()> {
    worth_signal::facade::SignalRuntime::builder(worth_signal::facade::SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

pub(crate) fn target_basis(
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
) -> &SignalBranchTarget {
    match basis.observation().target() {
        worth_foundational::FoundationalBranchTarget::Basis(target) => target,
        worth_foundational::FoundationalBranchTarget::Empty => panic!("Signal owner observations carry a basis target"),
    }
}