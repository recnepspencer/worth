use std::sync::Arc;

use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::owner::basis::map_basis_registry_denial;
use crate::branch::owner_services::{SignalOwner, SignalOwnerUnavailable};
use crate::branch::{SignalBranchBasisObservationDenial, SignalConditionalExecutionPort};
use crate::data::aspect::{Aspect, InstalledSignalScopedChangeSet};
use crate::data::conditional_execution::InstalledSignalConditionalContract;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::output::{CanonicalChangedRegions, ChangedRegion};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalCommittedPatchTarget {
    graph_instance_id: u64,
    node: NodeId,
    aspect: Aspect,
    changed_regions: CanonicalChangedRegions,
}

impl SignalCommittedPatchTarget {
    pub fn new(
        graph_instance_id: u64,
        node: NodeId,
        aspect: Aspect,
        changed_regions: impl IntoIterator<Item = ChangedRegion>,
    ) -> Self {
        Self {
            graph_instance_id,
            node,
            aspect,
            changed_regions: CanonicalChangedRegions::new(changed_regions),
        }
    }

    pub const fn graph_instance_id(&self) -> u64 {
        self.graph_instance_id
    }

    pub const fn node(&self) -> NodeId {
        self.node
    }

    pub const fn aspect(&self) -> Aspect {
        self.aspect
    }

    pub const fn changed_regions(&self) -> &CanonicalChangedRegions {
        &self.changed_regions
    }
}

#[derive(Debug)]
pub struct SignalCommittedPatchDeliveryRequest {
    targets: Vec<SignalCommittedPatchTarget>,
}

impl SignalCommittedPatchDeliveryRequest {
    pub fn new(targets: impl IntoIterator<Item = SignalCommittedPatchTarget>) -> Self {
        Self {
            targets: targets.into_iter().collect(),
        }
    }

    pub(crate) fn into_targets(self) -> Vec<SignalCommittedPatchTarget> {
        self.targets
    }
}

#[derive(Debug)]
pub enum SignalCommittedPatchDeliveryDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OwnerAdmission(SignalBranchBasisObservationDenial),
    StaleBasisAdmission,
    DefinitionReadmissionRequired,
    DefinitionMismatch,
    EmptyChangeSet,
    ForeignGraph,
    ForeignContractTarget,
    DuplicateTarget,
    MissingOrStaleTarget,
    SuccessorCaptureCapacityExhausted,
    SuccessorCaptureWorkExhausted { maximum_visits: usize },
    SuccessorCaptureUnavailable,
    SignalMutation(SignalError),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SignalCommittedPatchDeliveryCompletion {
    changes: InstalledSignalScopedChangeSet,
    transition: super::SignalConditionalSuccessorTransition,
}

impl SignalCommittedPatchDeliveryCompletion {
    pub(crate) const fn new(
        changes: InstalledSignalScopedChangeSet,
        transition: super::SignalConditionalSuccessorTransition,
    ) -> Self {
        Self {
            changes,
            transition,
        }
    }

    pub const fn graph_instance_id(&self) -> u64 {
        self.changes.graph_instance_id()
    }

    pub fn target_count(&self) -> usize {
        self.changes.len()
    }

    pub const fn changes(&self) -> &InstalledSignalScopedChangeSet {
        &self.changes
    }

    pub const fn successor_transition(&self) -> &super::SignalConditionalSuccessorTransition {
        &self.transition
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn deliver_committed_patch(
        &self,
        contract: &InstalledSignalConditionalContract,
        request: SignalCommittedPatchDeliveryRequest,
    ) -> Result<SignalCommittedPatchDeliveryCompletion, SignalCommittedPatchDeliveryDenial> {
        use SignalCommittedPatchDeliveryDenial as Denial;

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
        let prepared = super::SignalConditionalSuccessorTransition::reserve(
            &owner.conditional_retention,
            request.into_targets(),
        )
        .map_err(map_transition_reservation)?;
        cell.deliver_committed_patch(
            &admission,
            &self.basis,
            &self.definition,
            contract,
            &self._issuance_basis_custody,
            prepared,
            Arc::clone(&self.authority),
            &owner.conditional_retention,
        )
    }
}

fn map_transition_reservation(
    denial: crate::data::retained_storage::SignalConditionalRetentionDenial,
) -> SignalCommittedPatchDeliveryDenial {
    match denial {
        crate::data::retained_storage::SignalConditionalRetentionDenial::CapacityExhausted => {
            SignalCommittedPatchDeliveryDenial::SuccessorCaptureCapacityExhausted
        }
        crate::data::retained_storage::SignalConditionalRetentionDenial::Closed => {
            SignalCommittedPatchDeliveryDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        crate::data::retained_storage::SignalConditionalRetentionDenial::Poisoned
        | crate::data::retained_storage::SignalConditionalRetentionDenial::InvalidTransfer => {
            SignalCommittedPatchDeliveryDenial::SuccessorCaptureUnavailable
        }
    }
}
