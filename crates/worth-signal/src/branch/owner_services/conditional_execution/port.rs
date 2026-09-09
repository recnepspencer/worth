use std::sync::{Arc, Weak};

use crate::branch::AdmittedSignalBranchBasis;

use super::issuance::SignalConditionalServiceAuthority;
use super::SignalInstalledDefinitionBinding;
use crate::branch::owner_services::{SignalBranchCellIncarnation, SignalOwner};

/// Exact-basis conditional capability issued only to its definition claimant.
/// The capability retains its issuance basis as storage custody, but fresh
/// admissions snapshot the cell's current published basis.
/// It is separate from the ordinary three-port component service bundle.
pub struct SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) owner: Weak<SignalOwner<D, I, T>>,
    pub(super) basis: AdmittedSignalBranchBasis,
    pub(super) definition: SignalInstalledDefinitionBinding,
    pub(super) incarnation: SignalBranchCellIncarnation,
    pub(super) authority: Arc<SignalConditionalServiceAuthority>,
    pub(super) source_authority: worth_proof::ConditionalSourceObservationAuthority,
    pub(super) claimant: crate::data::aspect::SignalAspectLoweringOwner,
    pub(super) _issuance_basis_custody: Arc<super::SignalRetainedExecutionBasis>,
    pub(super) next_evaluation_ordinal: std::sync::atomic::AtomicU64,
    pub(super) next_installation_ordinal: std::sync::atomic::AtomicU64,
}

impl<D, I, T> std::fmt::Debug for SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SignalConditionalExecutionPort")
            .finish_non_exhaustive()
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn issuance_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.basis
    }

    pub fn retention_observation(
        &self,
    ) -> Result<
        super::SignalConditionalRetentionObservation,
        crate::branch::owner_services::SignalOwnerUnavailable,
    > {
        let owner = SignalOwner::upgrade(&self.owner)?;
        Ok(owner.conditional_retention.observe())
    }
}
