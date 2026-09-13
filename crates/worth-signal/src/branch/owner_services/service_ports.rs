use super::{SignalBranchBasisPort, SignalBranchLifecyclePort, SignalBranchMutationPort};

/// The three concrete independently borrowable services of one Signal owner.
pub struct SignalOwnerServicePorts<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    basis: SignalBranchBasisPort<D, I, T>,
    mutation: SignalBranchMutationPort<D, I, E, Ctx, T>,
    lifecycle: SignalBranchLifecyclePort<D, I, T>,
}

impl<D, I, E, Ctx, T> SignalOwnerServicePorts<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(
        basis: SignalBranchBasisPort<D, I, T>,
        mutation: SignalBranchMutationPort<D, I, E, Ctx, T>,
        lifecycle: SignalBranchLifecyclePort<D, I, T>,
    ) -> Self {
        Self {
            basis,
            mutation,
            lifecycle,
        }
    }

    pub fn basis_port(&self) -> SignalBranchBasisPort<D, I, T> {
        self.basis.clone()
    }

    pub fn mutation_port(&self) -> SignalBranchMutationPort<D, I, E, Ctx, T> {
        self.mutation.clone()
    }

    pub fn lifecycle_port(&self) -> SignalBranchLifecyclePort<D, I, T> {
        self.lifecycle.clone()
    }
}

impl<D, I, E, Ctx, T> Clone for SignalOwnerServicePorts<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn clone(&self) -> Self {
        Self {
            basis: self.basis.clone(),
            mutation: self.mutation.clone(),
            lifecycle: self.lifecycle.clone(),
        }
    }
}
