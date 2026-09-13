use std::sync::Arc;

use crate::branch::AdmittedSignalBranchBasis;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::logic::transaction::SignalRuntime;
use crate::state::SignalBranchHandle;

use super::super::super::{SignalBranchMutationPort, SignalOwner};

pub(super) type TestRuntime<Ctx> = SignalRuntime<(), (), (), Ctx, ()>;
pub(super) type TestPort<Ctx> = SignalBranchMutationPort<(), (), (), Ctx, ()>;

pub(super) struct MutationWorld<Ctx>
where
    Ctx: Send + Sync + 'static,
{
    pub(super) runtime: TestRuntime<Ctx>,
    pub(super) port: TestPort<Ctx>,
    pub(super) owner: Arc<SignalOwner<(), (), ()>>,
    pub(super) source_branch: SignalBranchHandle,
    pub(super) source_basis: AdmittedSignalBranchBasis,
    pub(super) sibling_basis: AdmittedSignalBranchBasis,
    pub(super) input_a: NodeId,
    pub(super) input_b: NodeId,
    pub(super) derived: NodeId,
}

impl<Ctx> MutationWorld<Ctx>
where
    Ctx: Send + Sync + 'static,
{
    pub(super) fn new() -> Self {
        let mut graph = SignalGraph::new();
        let input_a = graph.create_node();
        let input_b = graph.create_node();
        let derived = graph.create_node();
        graph
            .set_dependencies(derived, [DependencyEdge::new(input_a, Aspect::new(0))])
            .expect("semantic dependency installs");

        let mut runtime: TestRuntime<Ctx> = SignalRuntime::build_for::<Ctx>(graph);
        let source_branch = runtime.current_branch();
        let source_basis = runtime
            .observe_signal_branch_basis(source_branch.clone())
            .expect("the production runtime admits its source branch");
        let sibling_branch = runtime
            .fork_signal_branch("mutation-port-sibling", &source_basis)
            .expect("the production runtime forks a populated sibling before sealing")
            .created_branch()
            .clone();
        let sibling_basis = runtime
            .observe_signal_branch_basis(sibling_branch.clone())
            .expect("the production runtime admits its sibling branch");
        let (_, port, _) = runtime
            .owner_port_slots()
            .expect("the production runtime seals into owner cells");
        let owner = port
            .upgrade_owner()
            .expect("the runtime root keeps the sealed owner available");

        Self {
            runtime,
            port,
            owner,
            source_branch,
            source_basis,
            sibling_basis,
            input_a,
            input_b,
            derived,
        }
    }

    pub(super) fn dependency_sources(&self, branch: &SignalBranchHandle) -> Vec<NodeId> {
        let admission = self.owner.admit().expect("state observation admits");
        self.owner
            .lookup_cell(&admission, branch.id)
            .expect("the observed branch owns one canonical cell")
            .with_state(&admission, |state, _| {
                state
                    .state()
                    .graph()
                    .dependency_sources_of(self.derived)
                    .expect("the semantic derived node remains live")
            })
            .expect("the canonical state observation enters its cell")
    }

    pub(super) fn canonical_handle(&self, branch: &SignalBranchHandle) -> SignalBranchHandle {
        let admission = self.owner.admit().expect("handle observation admits");
        self.owner
            .lookup_cell(&admission, branch.id)
            .expect("the observed branch owns one canonical cell")
            .with_state(&admission, |state, _| state.handle().clone())
            .expect("the canonical handle observation enters its cell")
    }
}

pub(super) fn set_dependency<Ctx>(
    transaction: &mut crate::logic::transaction::SignalTransaction<'_, (), (), (), Ctx, ()>,
    derived: NodeId,
    source: NodeId,
) -> Result<(), crate::data::error::SignalError> {
    transaction.set_dependencies(derived, [DependencyEdge::new(source, Aspect::new(0))])
}
