//! Reconstructed declarations always start with fresh derived runtime state.
use super::{AuthorityState, BranchAncestryState, BranchMutationLedger, BranchState, DerivedState};
use crate::data::checkpoint_policy::CheckpointPolicy;
use crate::data::graph::SignalGraph;
use crate::logic::checkpoint::CheckpointRuntime;
use crate::logic::transaction::runtime::config::SignalRuntimeConfig;
use crate::state::SignalBranchHandle;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime) fn readmit_authored(
        graph: Box<SignalGraph>,
        config: SignalRuntimeConfig<T>,
        handle: &SignalBranchHandle,
        checkpoint_policy: CheckpointPolicy<D>,
    ) -> Box<Self> {
        Box::new(Self {
            authority: AuthorityState { graph, config },
            derived: Box::new(DerivedState {
                checkpoint: CheckpointRuntime::new(checkpoint_policy),
                resource: Default::default(),
                temporal: Default::default(),
                telemetry: Default::default(),
            }),
            ancestry: BranchAncestryState::new(
                handle.id,
                handle.parent_branch_id,
                handle.head_snapshot_id,
            ),
            mutation_ledger: BranchMutationLedger::default()
                .with_baseline_snapshot(handle.head_snapshot_id),
            installed_definition: None,
        })
    }
}
