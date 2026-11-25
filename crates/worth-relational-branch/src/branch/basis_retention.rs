use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::basis_denial::RelationalBranchBasisDenial;
use crate::history::retention::{
    RelationalBranchRetentionLease, RelationalBranchRetentionReleaseDenial,
    RelationalBranchRetentionReleaseReceipt,
};
use crate::internal_port::history::retention::lease::RelationalBranchRetentionReleaseDenialOwnerPort as _;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;

pub fn readmit_retained_branch_basis(
    history: &HistorySubsystem,
    runtime_id: u64,
    instrumentation: &RuntimeInstrumentation,
    descriptor: &crate::branch::basis::RelationalBranchBasisDescriptor,
    lease: &RelationalBranchRetentionLease,
) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
    let result = readmit_retained_branch_basis_inner(history, runtime_id, descriptor, lease);
    instrumentation.count_basis(|counters| counters.record_readmission(&result));
    result
}

fn readmit_retained_branch_basis_inner(
    history: &HistorySubsystem,
    runtime_id: u64,
    descriptor: &crate::branch::basis::RelationalBranchBasisDescriptor,
    lease: &RelationalBranchRetentionLease,
) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
    let descriptor =
        super::basis_descriptor_resolution::resolve_relational_branch_basis_descriptor(
            descriptor.clone(),
        )?
        .into_descriptor();
    if descriptor.runtime_instance_id() != runtime_id {
        return Err(RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id: runtime_id,
            actual_runtime_instance_id: descriptor.runtime_instance_id(),
        });
    }
    if lease.owner_relationship(&history.retention_binding())
        != crate::history::retention::RelationalRetentionOwnerRelationship::SameOwner
    {
        return Err(RelationalBranchBasisDenial::UnavailableRetainedTarget);
    }
    if lease.descriptor() != &descriptor {
        return Err(RelationalBranchBasisDenial::WrongImmutableTarget);
    }
    Ok(lease.admitted_basis())
}

/// Retain the same owner-issued immutable basis, including after its branch
/// retires. The original observation obligation establishes ownership;
/// a descriptor or a live branch name cannot substitute for it. This does
/// not establish currentness or admit a fork/publication.
/// A replaced retention owner cannot accept the previous owner's obligation.
pub fn retain_component_basis(
    history: &HistorySubsystem,
    runtime_id: u64,
    instrumentation: &RuntimeInstrumentation,
    basis: &AdmittedRelationalBranchBasis,
) -> Result<RelationalBranchRetentionLease, RelationalBranchBasisDenial> {
    if basis.identity().runtime_instance_id() != runtime_id {
        return Err(RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id: runtime_id,
            actual_runtime_instance_id: basis.identity().runtime_instance_id(),
        });
    }
    if basis
        .inner
        .retention
        .owner_relationship(&history.retention_binding())
        != crate::history::retention::RelationalRetentionOwnerRelationship::SameOwner
    {
        return Err(RelationalBranchBasisDenial::UnavailableRetainedTarget);
    }
    let lease = RelationalBranchRetentionLease::new(
        basis.observation(),
        &basis.inner.retention_binding,
        instrumentation.external_retention_terminal_accounting(),
    )
    .map_err(map_retention_acquisition_denial)?;
    instrumentation.count_basis(|counters| {
        counters.external_retention_acquires =
            counters.external_retention_acquires.saturating_add(1);
    });
    Ok(lease)
}

pub fn release_component_basis(
    history: &HistorySubsystem,
    runtime_id: u64,
    lease: RelationalBranchRetentionLease,
) -> Result<RelationalBranchRetentionReleaseReceipt, RelationalBranchRetentionReleaseDenial> {
    let retention_binding = history.retention_binding();
    if lease.descriptor().runtime_instance_id() != runtime_id {
        let denial = RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id: runtime_id,
            actual_runtime_instance_id: lease.descriptor().runtime_instance_id(),
        };
        return Err(RelationalBranchRetentionReleaseDenial::new(denial, lease));
    }
    match lease.owner_relationship(&retention_binding) {
        crate::history::retention::RelationalRetentionOwnerRelationship::SameOwner
        | crate::history::retention::RelationalRetentionOwnerRelationship::OwnerUnavailable => {
            Ok(lease.release())
        }
        crate::history::retention::RelationalRetentionOwnerRelationship::DifferentOwner => {
            Err(RelationalBranchRetentionReleaseDenial::new(
                RelationalBranchBasisDenial::UnavailableRetainedTarget,
                lease,
            ))
        }
    }
}

pub(super) fn map_retention_acquisition_denial(
    denial: crate::history::retention::RelationalRetentionAcquisitionDenial,
) -> RelationalBranchBasisDenial {
    match denial {
        crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
            RelationalBranchBasisDenial::RetentionCapacityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::OwnerUnavailable => {
            RelationalBranchBasisDenial::UnavailableRetainedTarget
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
            RelationalBranchBasisDenial::RetentionIdentityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::RootSetTooLarge
        | crate::history::retention::RelationalRetentionAcquisitionDenial::BindingMismatch => {
            RelationalBranchBasisDenial::OwnerFailure
        }
    }
}
