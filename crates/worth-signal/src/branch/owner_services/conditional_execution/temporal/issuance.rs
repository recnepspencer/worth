use std::num::NonZeroUsize;
use std::sync::Arc;

use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::owner::basis::map_basis_registry_denial;
use crate::branch::owner_services::SignalOwner;

use super::super::SignalConditionalExecutionPort;
use super::{
    SignalConditionalTemporalPartition, SignalConditionalTemporalPartitionDenial as Denial,
};

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Register an independent bounded clock partition on this exact sealed owner.
    pub fn admit_temporal_partition(
        &self,
        maximum_active_wakes: NonZeroUsize,
    ) -> Result<SignalConditionalTemporalPartition<D, I, T>, Denial> {
        let owner = SignalOwner::upgrade(&self.owner).map_err(Denial::OwnerUnavailable)?;
        let admission = owner
            .admit()
            .map_err(map_observation_admission_denial)
            .map_err(Denial::OwnerAdmission)?;
        let branch = self.basis.owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, branch)
            .map_err(|denial| Denial::OwnerAdmission(map_basis_registry_denial(denial, branch)))?;
        if cell.incarnation() != self.incarnation {
            return Err(Denial::StaleBasisAdmission);
        }
        cell.admit_current_conditional_service(&admission, &self.basis, &self.definition)
            .map_err(Denial::ServiceAdmission)?;
        if !owner.is_current_canonical_basis(
            &self.basis,
            branch,
            self.incarnation.get(),
            self.basis.observation(),
        ) {
            return Err(Denial::StaleBasisAdmission);
        }
        let (id, cell, custody) = owner.conditional_temporal.register(
            &admission,
            maximum_active_wakes,
            &owner.conditional_retention,
        )?;
        Ok(SignalConditionalTemporalPartition {
            owner: Arc::downgrade(&owner),
            cell,
            id,
            basis: self.basis.clone(),
            owner_runtime_instance_id: owner.runtime_instance_id(),
            _custody: custody,
        })
    }
}
