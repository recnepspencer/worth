use super::{
    basis_identity_validation::require_local_branch_identity, AdmittedRelationalBranchBasis,
    RelationalBranchBasisDenial,
};
use crate::history::retention::{
    RelationalBranchRetentionLease, RelationalBranchRetentionReleaseDenial,
    RelationalBranchRetentionReleaseReceipt,
};
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub fn readmit_retained_branch_basis(
        &self,
        descriptor: &crate::branch::RelationalBranchBasisDescriptor,
        lease: &RelationalBranchRetentionLease,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        let result = self.readmit_retained_branch_basis_inner(descriptor, lease);
        self.services
            .instrumentation
            .count_basis(|counters| counters.record_readmission(&result));
        result
    }

    fn readmit_retained_branch_basis_inner(
        &self,
        descriptor: &crate::branch::RelationalBranchBasisDescriptor,
        lease: &RelationalBranchRetentionLease,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        let descriptor =
            super::basis_descriptor_resolution::resolve_relational_branch_basis_descriptor(
                descriptor.clone(),
            )?
            .into_descriptor();
        if descriptor.runtime_instance_id() != self.runtime_instance_id() {
            return Err(RelationalBranchBasisDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime_instance_id(),
                actual_runtime_instance_id: descriptor.runtime_instance_id(),
            });
        }
        if lease.owner_relationship(&self.history.retention_binding())
            != crate::history::retention::RelationalRetentionOwnerRelationship::SameOwner
        {
            return Err(RelationalBranchBasisDenial::UnavailableRetainedTarget);
        }
        if lease.descriptor() != &descriptor {
            return Err(RelationalBranchBasisDenial::WrongImmutableTarget);
        }
        Ok(lease.admitted_basis())
    }

    pub fn retain_component_basis(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<RelationalBranchRetentionLease, RelationalBranchBasisDenial> {
        require_local_branch_identity(self, basis.identity())?;
        let lease = RelationalBranchRetentionLease::new(
            basis.observation(),
            &basis.inner.retention_binding,
            self.services
                .instrumentation
                .external_retention_terminal_accounting(),
        )
        .map_err(map_retention_acquisition_denial)?;
        self.services.instrumentation.count_basis(|counters| {
            counters.external_retention_acquires =
                counters.external_retention_acquires.saturating_add(1);
        });
        Ok(lease)
    }

    pub fn release_component_basis(
        &self,
        lease: RelationalBranchRetentionLease,
    ) -> Result<RelationalBranchRetentionReleaseReceipt, RelationalBranchRetentionReleaseDenial>
    {
        let retention_binding = self.history.retention_binding();
        if lease.descriptor().runtime_instance_id() != self.runtime_instance_id() {
            let denial = RelationalBranchBasisDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime_instance_id(),
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
}

fn map_retention_acquisition_denial(
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
