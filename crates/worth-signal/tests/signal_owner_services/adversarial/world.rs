use worth_signal::facade::branch::{
    validate_signal_branch_name, AdmittedSignalBranchBasis, SignalBranchBasisPort,
    SignalBranchLifecyclePort, SignalBranchMutationPort, SignalOwnerCancellationSource,
};
use worth_signal::facade::{Aspect, DependencyEdge, SignalGraph, SignalRuntime};

pub(super) type Runtime = SignalRuntime<(), (), (), (), ()>;
pub(super) type BasisPort = SignalBranchBasisPort<(), (), ()>;
pub(super) type MutationPort = SignalBranchMutationPort<(), (), (), (), ()>;
pub(super) type LifecyclePort = SignalBranchLifecyclePort<(), (), ()>;

pub(super) const PROGRESS_BOUND: std::time::Duration = std::time::Duration::from_secs(3);

/// A real owner with two admitted branches and all three weak public ports.
///
/// The runtime is optional so a test can request root destruction while it
/// retains weak ports, exact bases, and any operation-control handle.
pub(super) struct AdversarialWorld {
    pub(super) runtime: Option<Runtime>,
    pub(super) basis: BasisPort,
    pub(super) mutation: MutationPort,
    pub(super) lifecycle: LifecyclePort,
    pub(super) root_basis: AdmittedSignalBranchBasis,
    pub(super) child_basis: AdmittedSignalBranchBasis,
}

impl AdversarialWorld {
    pub(super) fn new() -> Self {
        let mut runtime = Runtime::builder(populated_graph())
            .with_kernel_defaults()
            .build();
        let root_basis = runtime
            .observe_signal_branch_basis(runtime.current_branch())
            .expect("the bootstrap branch is owner-admitted before sealing");
        let services = runtime
            .owner_component_services()
            .expect("the real owner seals and issues weak public ports");
        let basis = services.basis_port();
        let mutation = services.mutation_port();
        let lifecycle = services.lifecycle_port();
        let child_basis = mutation
            .fork_exact(
                validate_signal_branch_name("adversarial-child")
                    .expect("the fixture name is a valid semantic identity"),
                &root_basis,
                &SignalOwnerCancellationSource::new().token(),
            )
            .expect("the real owner creates a second admitted branch")
            .into_parts()
            .1;

        Self {
            runtime: Some(runtime),
            basis,
            mutation,
            lifecycle,
            root_basis,
            child_basis,
        }
    }

    pub(super) fn close_root(&mut self) {
        self.runtime.take();
    }
}

fn populated_graph() -> SignalGraph {
    let mut graph = SignalGraph::new();
    let input = graph.node().produces_aspects(Aspect::new(0)).build();
    let derived = graph
        .node()
        .on_demand()
        .produces_aspects(Aspect::new(1))
        .build();
    graph
        .set_dependencies(derived, [DependencyEdge::new(input, Aspect::new(0))])
        .expect("the adversarial graph has one valid dependency");
    graph
}
