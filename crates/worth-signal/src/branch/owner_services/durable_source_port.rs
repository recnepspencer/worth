//! Weak access to authored source under the real sealed owner and branch cells.
mod recorded_point;
use super::{SignalDurableBranchHandle, SignalOwner, SignalOwnerUnavailable};
use crate::branch::{
    SignalBranchBasisObservationDenial, SignalBranchBasisReadmissionDenial,
    SignalBranchRetentionAcquisitionDenial,
};
use crate::durability::{
    SignalAuthoredImage, SignalAuthoredImageDenial, SignalDurableBranchDescriptor,
    SignalDurableFrameDenial, SignalDurableTierVocabulary, SignalRuntimeAuthoredImage,
};
use crate::state::{SignalBranchHandle, SignalBranchId};
use std::sync::{Arc, Weak};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{DurableSignalBranchIdentity, DurableSignalOwnerIdentity};

#[derive(Debug)]
pub enum SignalDurableSourceDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    Admission(SignalBranchBasisReadmissionDenial),
    Observation(SignalBranchBasisObservationDenial),
    Retention(SignalBranchRetentionAcquisitionDenial),
    Frame(SignalDurableFrameDenial),
    Image(SignalAuthoredImageDenial),
    HomeIncarnation(worth_proof::HomeIncarnationDenial),
    ForeignOwner,
    ForeignBasis,
    DescriptorMismatch,
    SourceNotInstalled,
    CatalogUnavailable,
    CatalogReentry,
    CatalogOrdering,
    CatalogInvariantViolation,
}
pub struct SignalDurableBranchSourcePort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: Weak<SignalOwner<D, I, T>>,
    identity: DurableSignalOwnerIdentity,
    fence: worth_proof::HomeIncarnationFence,
    vocabulary: Arc<SignalDurableTierVocabulary<T>>,
}
impl<D, I, T> Clone for SignalDurableBranchSourcePort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            identity: self.identity,
            fence: self.fence,
            vocabulary: Arc::clone(&self.vocabulary),
        }
    }
}
impl<D, I, T> SignalDurableBranchSourcePort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(
        owner: Weak<SignalOwner<D, I, T>>,
        binding: &crate::durability::SignalDurableHomeBinding,
        vocabulary: Arc<SignalDurableTierVocabulary<T>>,
    ) -> Self {
        Self {
            owner,
            identity: binding.owner(),
            fence: binding.incarnation().fence(),
            vocabulary,
        }
    }
    fn owner(&self) -> Result<Arc<SignalOwner<D, I, T>>, SignalDurableSourceDenial> {
        SignalOwner::upgrade(&self.owner).map_err(SignalDurableSourceDenial::OwnerUnavailable)
    }
    pub fn branches(
        &self,
    ) -> Result<Vec<(DurableSignalBranchIdentity, SignalBranchHandle)>, SignalDurableSourceDenial>
    {
        use SignalDurableSourceDenial as D;
        let owner = self.owner()?;
        let admission = owner
            .admit()
            .map_err(super::basis_port::denial_mapping::map_basis_admission_denial)
            .map_err(D::Admission)?;
        let cells = owner
            .registry()
            .live_cells_in_identity_order(&admission)
            .map_err(|denial| {
                use super::SignalBranchRegistryDenial as R;
                match denial {
                    R::ForeignOwner | R::ExpiredAdmission => D::CatalogUnavailable,
                    R::OwnerReentry => D::CatalogReentry,
                    R::OwnerMetadataOrdering => D::CatalogOrdering,
                    R::DuplicateBranch(_)
                    | R::UnknownBranch(_)
                    | R::LiveCapacityExhausted { .. }
                    | R::ReservationCapacityExhausted { .. }
                    | R::NameAlreadyReserved
                    | R::NameAlreadyInstalled
                    | R::RetirementInProgress(_)
                    | R::ExpiredRetirement(_) => D::CatalogInvariantViolation,
                    #[cfg(test)]
                    R::TargetCellDenied(_) => D::CatalogInvariantViolation,
                }
            })?;
        let mut branches = Vec::with_capacity(cells.len());
        for cell in cells {
            let handle = cell
                .descriptive_handle(&admission)
                .map_err(|denial| {
                    super::branch_execution_cell::basis::map_basis_cell_denial(
                        denial,
                        cell.branch_id(),
                    )
                })
                .map_err(D::Observation)?;
            branches.push((
                DurableSignalBranchIdentity::from_recorded(self.identity, handle.id.0),
                handle,
            ));
        }
        Ok(branches)
    }
    pub fn observe(
        &self,
        branch: DurableSignalBranchIdentity,
    ) -> Result<SignalDurableBranchDescriptor, SignalDurableSourceDenial> {
        use SignalDurableSourceDenial as D;
        if branch.owner() != self.identity {
            return Err(D::ForeignOwner);
        }
        let owner = self.owner()?;
        let admission = owner
            .admit()
            .map_err(super::basis_port::denial_mapping::map_basis_admission_denial)
            .map_err(D::Admission)?;
        let id = SignalBranchId(branch.ordinal());
        let cell = owner
            .lookup_cell(&admission, id)
            .map_err(|denial| super::owner::basis::map_basis_registry_denial(denial, id))
            .map_err(D::Observation)?;
        cell.observe_durable_source(&admission)
            .map(|(point, _)| point.descriptor().clone())
    }
    pub fn readmit_descriptor(
        &self,
        descriptor: &SignalDurableBranchDescriptor,
    ) -> Result<SignalDurableBranchHandle, SignalDurableSourceDenial> {
        use SignalDurableSourceDenial as D;
        if descriptor.owner() != self.identity {
            return Err(D::ForeignOwner);
        }
        let owner = self.owner()?;
        let admission = owner
            .admit()
            .map_err(super::basis_port::denial_mapping::map_basis_admission_denial)
            .map_err(D::Admission)?;
        let id = SignalBranchId(descriptor.branch().ordinal());
        let cell = owner
            .lookup_cell(&admission, id)
            .map_err(|denial| super::owner::basis::map_basis_registry_denial(denial, id))
            .map_err(D::Observation)?;
        let (actual, observation) = cell.observe_durable_source(&admission)?;
        if actual.descriptor() != descriptor {
            return Err(D::DescriptorMismatch);
        }
        let basis = owner
            .admit_canonical_basis_with_retention(
                &admission,
                observation,
                id,
                cell.incarnation().get(),
                || owner.acquire_admitted_retention(&admission, id),
            )
            .map_err(D::Retention)?;
        Ok(SignalDurableBranchHandle::from_owner(
            basis, actual, self.fence,
        ))
    }
    pub fn readmit_descriptor_bytes(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        bytes: &[u8],
    ) -> Result<SignalDurableBranchHandle, SignalDurableSourceDenial> {
        let descriptor = SignalDurableBranchDescriptor::readmit_bytes(execution, bytes)
            .map_err(SignalDurableSourceDenial::Frame)?;
        self.readmit_descriptor(&descriptor)
    }
    pub fn capture_authored_images(
        &self,
        branch: &SignalDurableBranchHandle,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(SignalAuthoredImage, SignalRuntimeAuthoredImage), SignalDurableSourceDenial> {
        use SignalDurableSourceDenial as D;
        if branch.descriptor().owner() != self.identity {
            return Err(D::ForeignOwner);
        }
        self.fence
            .admit(branch.fence())
            .map_err(D::HomeIncarnation)?;
        let basis = branch.basis();
        let owner = self.owner()?;
        if !owner.basis_has_owner_affinity(basis) {
            return Err(D::ForeignBasis);
        }
        let admission = owner
            .admit()
            .map_err(super::basis_port::denial_mapping::map_basis_admission_denial)
            .map_err(D::Admission)?;
        let id = basis.owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, id)
            .map_err(|denial| super::owner::basis::map_basis_registry_denial(denial, id))
            .map_err(D::Observation)?;
        cell.capture_durable_source(&owner, &admission, basis, &self.vocabulary, execution)
    }
}
