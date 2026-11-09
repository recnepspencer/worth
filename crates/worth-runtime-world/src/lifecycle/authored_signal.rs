//! One actual authored Signal owner is bound to one composing World issuer.
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{DurableSignalOwnerIdentity, HomeIncarnation};
use worth_proof::InstalledCompositeSignalPublicationOwner;
use worth_signal::facade::{
    branch::SignalBranchBasisPort,
    durability::{
        SignalCompositeOwnerServicePorts, SignalDurableAdvanceDenial,
        SignalDurableBranchSourcePort, SignalDurableOwner,
    },
};

/// Retain this SAME attachment across Home construction retries. Services are
/// weak to the actual durable Signal root; this handle never creates a graph.
pub struct RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(super) services: SignalCompositeOwnerServicePorts<D, I, E, Ctx, T>,
    pub(super) issuer: InstalledCompositeSignalPublicationOwner,
    identity: DurableSignalOwnerIdentity,
    incarnation: HomeIncarnation,
    _funding: Option<ExecutionMemoryReservation>,
}
#[derive(Debug)]
pub enum RuntimeWorldAuthoredSignalAttachmentDenial {
    Request(worth_execution::WorkCeilingDenial),
    Signal(Box<SignalDurableAdvanceDenial>),
    ForeignHome,
    StaleIncarnation,
}
impl<D, I, E, Ctx, T> RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub fn install(
        owner: &SignalDurableOwner<D, I, E, Ctx, T>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, RuntimeWorldAuthoredSignalAttachmentDenial> {
        use RuntimeWorldAuthoredSignalAttachmentDenial as Denial;
        // Quote the complete Arc allocation before installing its affinity.
        let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed authored attachment layout")
            .0
            .pad_to_align();
        let funding = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(Denial::Request)?;
        let issuer = InstalledCompositeSignalPublicationOwner::fresh();
        let services = owner
            .bind_composite_owner(issuer.authority(), request)
            .map_err(|cause| Denial::Signal(Box::new(cause)))?;
        Ok(Arc::new(Self {
            services,
            issuer,
            identity: owner.binding().owner(),
            incarnation: owner.binding().incarnation(),
            _funding: funding,
        }))
    }
    pub fn basis_port(&self) -> SignalBranchBasisPort<D, I, T> {
        self.services.basis_port()
    }
    pub fn source_port(&self) -> SignalDurableBranchSourcePort<D, I, T> {
        self.services.source_port()
    }
    pub(super) fn recorded_current_basis(
        &self,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<
        worth_signal::facade::durability::SignalRecordedSourcePoint,
        worth_signal::facade::durability::SignalDurableSourceDenial,
    > {
        use worth_signal::facade::durability::SignalDurableSourceDenial as Denial;
        let source = self.services.source_port();
        let branch = worth_foundational::facade::DurableSignalBranchIdentity::from_recorded(
            self.identity,
            basis.branch_id().0,
        );
        let descriptor = source.observe(branch)?;
        let actual = source.readmit_descriptor(&descriptor)?;
        if actual.basis().admission_identity() != basis.admission_identity() {
            return Err(Denial::ForeignBasis);
        }
        Ok(actual.source_point().clone())
    }
    pub(super) fn admit_home(
        &self,
        binding: &crate::durability::WorldDurableHomeBinding,
    ) -> Result<(), RuntimeWorldAuthoredSignalAttachmentDenial> {
        use RuntimeWorldAuthoredSignalAttachmentDenial as Denial;
        if self.identity.home() != binding.owner().home() {
            return Err(Denial::ForeignHome);
        }
        if self.incarnation != binding.incarnation() {
            return Err(Denial::StaleIncarnation);
        }
        Ok(())
    }
}
