pub(crate) use {
    worth_signal::facade::branch::validate_signal_branch_name,
    worth_signal::facade::branch::AdmittedSignalBranchBasis,
    worth_signal::facade::branch::SignalBranchBasisPort,
    worth_signal::facade::branch::SignalBranchLifecyclePort,
    worth_signal::facade::branch::SignalBranchMutationPort,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
};
pub(crate) use {
    worth_signal::facade::Aspect, worth_signal::facade::DependencyEdge,
    worth_signal::facade::SignalGraph, worth_signal::facade::SignalRuntime,
};

pub(crate) type Runtime = SignalRuntime<(), (), (), (), ()>;
pub(crate) type BasisPort = SignalBranchBasisPort<(), (), ()>;
pub(crate) type MutationPort = SignalBranchMutationPort<(), (), (), (), ()>;
pub(crate) type LifecyclePort = SignalBranchLifecyclePort<(), (), ()>;

pub(crate) const PROGRESS_BOUND: std::time::Duration = std::time::Duration::from_secs(3);

pub(crate) struct AdversarialWorld {
    pub(crate) runtime: Option<Runtime>,
    pub(crate) basis: BasisPort,
    pub(crate) mutation: MutationPort,
    pub(crate) lifecycle: LifecyclePort,
    pub(crate) root_basis: AdmittedSignalBranchBasis,
    pub(crate) child_basis: AdmittedSignalBranchBasis,
}

impl AdversarialWorld {
    pub(crate) fn new() -> Self {
        let mut runtime =
            crate::signal_owner_services::ordinary_context::adversarial::world::Runtime::builder(
                populated_graph(),
            )
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
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
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

    pub(crate) fn close_root(&mut self) {
        self.runtime.take();
    }
}

pub(crate) fn populated_graph() -> SignalGraph {
    let mut graph = worth_signal::facade::SignalGraph::new();
    let input = graph
        .node()
        .produces_aspects(worth_signal::facade::Aspect::new(0))
        .build();
    let derived = graph
        .node()
        .on_demand()
        .produces_aspects(worth_signal::facade::Aspect::new(1))
        .build();
    graph
        .set_dependencies(
            derived,
            [worth_signal::facade::DependencyEdge::new(
                input,
                worth_signal::facade::Aspect::new(0),
            )],
        )
        .expect("the adversarial graph has one valid dependency");
    graph
}
