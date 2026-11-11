//! Composite attachment exposes the actual durable owner without a memory writer.
use crate::branch::owner_services::{
    SignalBranchBasisPort, SignalDurableBranchMutationPort, SignalDurableBranchSourcePort,
};

pub struct SignalCompositeOwnerServicePorts<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(super) basis: SignalBranchBasisPort<D, I, T>,
    pub(super) source: SignalDurableBranchSourcePort<D, I, T>,
    pub(super) mutation: SignalDurableBranchMutationPort<D, I, E, Ctx, T>,
}
impl<D, I, E, Ctx, T> SignalCompositeOwnerServicePorts<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub fn basis_port(&self) -> SignalBranchBasisPort<D, I, T> {
        self.basis.clone()
    }
    pub fn source_port(&self) -> SignalDurableBranchSourcePort<D, I, T> {
        self.source.clone()
    }
    pub fn mutation_port(&self) -> SignalDurableBranchMutationPort<D, I, E, Ctx, T> {
        self.mutation.clone()
    }
}
