use std::sync::Arc;

use crate::branch::owner_services::conditional_execution::{
    SignalConditionalRetentionLedger, SignalInstalledDefinitionBinding,
};
use crate::branch::owner_services::SignalBranchCellIncarnation;
use crate::branch::SignalCommittedPatchTarget;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, SignalConditionalRetentionDenial,
    SignalConditionalRetentionReservation,
};

use super::super::{SignalConditionalServiceAuthority, SignalRetainedExecutionBasis};

/// Opaque evidence for one exact owner-published conditional basis transition.
/// It grants successor readmission only and has no delivery operation.
#[derive(Clone)]
pub struct SignalConditionalSuccessorTransition {
    pub(in crate::branch::owner_services) inner: Arc<SignalConditionalSuccessorTransitionInner>,
}

pub(in crate::branch::owner_services) struct SignalConditionalSuccessorTransitionInner {
    pub(in crate::branch::owner_services) service_authority: Arc<SignalConditionalServiceAuthority>,
    pub(in crate::branch::owner_services) definition: SignalInstalledDefinitionBinding,
    pub(in crate::branch::owner_services) incarnation: SignalBranchCellIncarnation,
    pub(in crate::branch::owner_services) predecessor: Arc<SignalRetainedExecutionBasis>,
    pub(in crate::branch::owner_services) successor: Arc<SignalRetainedExecutionBasis>,
    pub(in crate::branch::owner_services) targets: Arc<[SignalCommittedPatchTarget]>,
    pub(in crate::branch::owner_services) _custody: SignalConditionalRetentionReservation,
}

impl std::fmt::Debug for SignalConditionalSuccessorTransition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SignalConditionalSuccessorTransition")
            .field("target_count", &self.inner.targets.len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for SignalConditionalSuccessorTransition {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for SignalConditionalSuccessorTransition {}

impl SignalConditionalSuccessorTransition {
    pub(in crate::branch::owner_services) fn reserve(
        ledger: &Arc<SignalConditionalRetentionLedger>,
        targets: Vec<SignalCommittedPatchTarget>,
    ) -> Result<SignalPreparedConditionalTransition, SignalConditionalRetentionDenial> {
        let targets: Arc<[SignalCommittedPatchTarget]> = targets.into();
        let mut charge = arc_allocation_charge::<SignalConditionalSuccessorTransitionInner>()
            .and_then(|charge| charge.checked_add(target_charge(&targets)))
            .map_err(|_| SignalConditionalRetentionDenial::CapacityExhausted)?;
        // The returned reservation handle lives inline in the retained Arc.
        charge = charge
            .checked_add(
                Charge::capacity::<SignalConditionalRetentionReservation>(1)
                    .map_err(|_| SignalConditionalRetentionDenial::CapacityExhausted)?,
            )
            .map_err(|_| SignalConditionalRetentionDenial::CapacityExhausted)?;
        let custody = ledger.reserve(0, charge)?;
        Ok(SignalPreparedConditionalTransition { targets, custody })
    }

    pub(in crate::branch::owner_services) fn predecessor_is(
        &self,
        basis: &Arc<SignalRetainedExecutionBasis>,
    ) -> bool {
        Arc::ptr_eq(&self.inner.predecessor, basis)
    }
}

pub(in crate::branch::owner_services) struct SignalPreparedConditionalTransition {
    targets: Arc<[SignalCommittedPatchTarget]>,
    custody: SignalConditionalRetentionReservation,
}

impl SignalPreparedConditionalTransition {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::branch::owner_services) fn complete(
        self,
        service_authority: Arc<SignalConditionalServiceAuthority>,
        definition: SignalInstalledDefinitionBinding,
        incarnation: SignalBranchCellIncarnation,
        predecessor: Arc<SignalRetainedExecutionBasis>,
        successor: Arc<SignalRetainedExecutionBasis>,
    ) -> SignalConditionalSuccessorTransition {
        SignalConditionalSuccessorTransition {
            inner: Arc::new(SignalConditionalSuccessorTransitionInner {
                service_authority,
                definition,
                incarnation,
                predecessor,
                successor,
                targets: self.targets,
                _custody: self.custody,
            }),
        }
    }

    pub(in crate::branch::owner_services) fn targets(&self) -> &[SignalCommittedPatchTarget] {
        &self.targets
    }
}

fn target_charge(targets: &[SignalCommittedPatchTarget]) -> Charge {
    let mut bytes = std::mem::size_of_val(targets) as u64;
    for target in targets {
        for region in target.changed_regions().as_slice() {
            bytes = bytes
                .saturating_add(region.partition.0.capacity() as u64)
                .saturating_add(region.detail.as_ref().map_or(0, String::capacity) as u64);
        }
    }
    // Saturation is rejected by the ledger if it exceeds its configured bound.
    Charge::from_bytes(bytes)
}
