use super::super::branching::BranchState;
use crate::state::SignalBranchId;
#[derive(Debug)]
pub(in crate::logic::transaction::runtime) struct AuthorityTransferPacket<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    branch_id: SignalBranchId,
    state: BranchState<D, I, T>,
}

impl<D, I, T> AuthorityTransferPacket<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn new(branch_id: SignalBranchId, state: BranchState<D, I, T>) -> Self {
        Self { branch_id, state }
    }

    pub fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }

    pub fn into_state(self) -> BranchState<D, I, T> {
        self.state
    }

    pub fn state(&self) -> &BranchState<D, I, T> {
        &self.state
    }
}

#[derive(Debug)]
pub(in crate::logic::transaction::runtime) struct RestoreTransferPacket<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    branch_id: SignalBranchId,
    state: BranchState<D, I, T>,
}

impl<D, I, T> RestoreTransferPacket<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn new(branch_id: SignalBranchId, state: BranchState<D, I, T>) -> Self {
        Self { branch_id, state }
    }

    pub fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }

    pub fn into_state(self) -> BranchState<D, I, T> {
        self.state
    }

    pub fn state(&self) -> &BranchState<D, I, T> {
        &self.state
    }
}

#[derive(Debug)]
pub(in crate::logic::transaction::runtime) struct ExplicitBranchForkPacket<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    source_branch: SignalBranchId,
    branch_id: SignalBranchId,
    state: BranchState<D, I, T>,
}

impl<D, I, T> ExplicitBranchForkPacket<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn new(
        source_branch: SignalBranchId,
        branch_id: SignalBranchId,
        state: BranchState<D, I, T>,
    ) -> Self {
        Self {
            source_branch,
            branch_id,
            state,
        }
    }

    pub fn source_branch(&self) -> SignalBranchId {
        self.source_branch
    }

    pub fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }

    pub fn into_state(self) -> BranchState<D, I, T> {
        self.state
    }

    pub fn state(&self) -> &BranchState<D, I, T> {
        &self.state
    }
}

#[derive(Debug)]
pub(in crate::logic::transaction::runtime) enum BranchLifecycleTransfer<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    Move(AuthorityTransferPacket<D, I, T>),
    Restore(RestoreTransferPacket<D, I, T>),
}
