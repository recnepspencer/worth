use crate::diagnostics::{ReplayView, SynthesizedLineageChain};
use crate::state::{SignalBranchHandle, SignalBranchId};

use super::runtime_state::SignalRuntime;

pub struct RuntimeHistory<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime: &'a mut SignalRuntime<D, I, E, Ctx, T>,
}

impl<'a, D, I, E, Ctx, T> RuntimeHistory<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(runtime: &'a mut SignalRuntime<D, I, E, Ctx, T>) -> Self {
        Self { runtime }
    }

    pub fn current_branch(&self) -> SignalBranchHandle {
        self.runtime.current_branch()
    }

    pub fn branches(&self) -> Vec<SignalBranchHandle> {
        self.runtime.known_branches()
    }

    pub fn branch(&self, branch_id: SignalBranchId) -> Option<SignalBranchHandle> {
        self.runtime.branch_handle(branch_id)
    }

    pub fn ancestry(&self, branch_id: SignalBranchId) -> Vec<SignalBranchHandle> {
        self.runtime.branch_ancestry(branch_id)
    }

    pub fn replay_for_branch(&self, branch_id: SignalBranchId) -> ReplayView {
        self.runtime.replay_for_branch(branch_id)
    }

    pub fn replay_for_node(&self, node: crate::data::handle::NodeId) -> ReplayView {
        self.runtime.observe().replay_for_node(node)
    }

    pub fn lineage_for_node(&self, node: crate::data::handle::NodeId) -> SynthesizedLineageChain {
        self.runtime.observe().lineage_chain_for_node(node)
    }

    pub fn latest_lineage(&self) -> crate::diagnostics::lineage::RetainedLineageView<'_> {
        self.runtime.graph().observe().lineage_records()
    }
}

#[cfg(test)]
#[cfg(test)]
use crate::data::aspect::Aspect;
