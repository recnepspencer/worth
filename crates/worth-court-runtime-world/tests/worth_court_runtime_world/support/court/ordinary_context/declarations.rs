#[allow(unused_imports)]
pub(crate) use {};

#[allow(unused_imports)]
pub(crate) use crate::cost::support::reference_context;

#[allow(unused_imports)]
pub(crate) use crate::court::reference_context::model;

#[allow(unused_imports)]
pub(crate) use {};

pub(crate) use crate::court::ordinary_context::relational::CargoRecords;
pub(crate) use worth_runtime_world::facade::*;
pub(crate) use worth_signal::facade::SignalGraph;
pub(crate) use {
    crate::court::ordinary_context::routing::CargoEvent,
    crate::court::ordinary_context::routing::CargoKey,
    crate::court::ordinary_context::routing::CourtSignal,
    crate::court::ordinary_context::routing::RouteDefinition,
    crate::court::ordinary_context::routing::RouteGraph,
    crate::court::ordinary_context::routing::RoutingContext,
};
pub(crate) use {
    worth_runtime_bridge::facade::BridgeInstalledSemanticCorrespondence,
    worth_runtime_bridge::facade::RuntimeBridge,
};

pub(crate) struct CompositeSupplyChainCourt {
    pub(crate) world: RuntimeWorldOwner<RouteDefinition, CargoKey, CargoEvent, RoutingContext, u64>,
    pub(crate) records: CargoRecords,
    pub(crate) signal: CourtSignal,
    pub(crate) bridge: RuntimeBridge,
    pub(crate) installed: BridgeInstalledSemanticCorrespondence,
    pub(crate) nodes: RouteGraph,
    pub(crate) initial: RuntimeWorldBootstrapIntent,
}
impl CompositeSupplyChainCourt {
    pub(crate) fn compile() -> Self {
        Self::compile_with_steel(true)
    }
    pub(crate) fn compile_with_steel(include_steel: bool) -> Self {
        Self::compile_config(
            include_steel,
            crate::court::ordinary_context::budgets::court(),
        )
    }
    pub(crate) fn compile_config(include_steel: bool, limits: RuntimeWorldBudgets) -> Self {
        let records =
            crate::court::ordinary_context::relational::CargoRecords::install(include_steel);
        let mut graph = worth_signal::facade::SignalGraph::new();
        let nodes = crate::court::ordinary_context::routing::RouteGraph::install(&mut graph);
        let (bridge, installed, _) =
            crate::court::ordinary_context::correspondence::install(&records, &mut graph, nodes);
        Self::from_installed_graph(records, graph, nodes, bridge, installed, limits)
    }
    pub(crate) fn from_installed_graph(
        records: CargoRecords,
        graph: SignalGraph,
        nodes: RouteGraph,
        bridge: RuntimeBridge,
        installed: BridgeInstalledSemanticCorrespondence,
        limits: RuntimeWorldBudgets,
    ) -> Self {
        let mut signal = worth_signal::facade::SignalRuntime::builder(graph)
            .with_domains::<RouteDefinition>()
            .with_impacts::<CargoKey>()
            .with_events::<CargoEvent>()
            .with_context::<RoutingContext>()
            .with_tiers::<u64>()
            .with_kernel_defaults()
            .build();
        let signal_basis = signal
            .observe_signal_branch_basis(signal.current_branch())
            .expect("component owner: initial Signal basis");
        let services = signal
            .owner_component_services()
            .expect("component owner: seal actual graph");
        let signal_definition_publication = signal
            .runtime_world_definition_publication_port()
            .expect("component owner: dedicate definition publication to World");
        let reference = services
            .basis_port()
            .issue_managed_branch_reference(&signal_basis)
            .unwrap();
        let signal_basis = services
            .basis_port()
            .readmit_exact(&reference, signal_basis.descriptor())
            .expect("component owner: initial exact basis through sealed services");
        let bridge_port = bridge.runtime_world_correspondence_port();
        let correspondence = bridge_port
            .admit_installed_basis(&installed)
            .expect("component owner: admit installed witness");
        let relational_basis = records
            .runtime
            .observe_branch(&records.runtime.main_branch_identity())
            .unwrap()
            .1;
        let initial = RuntimeWorldBootstrapIntent::new(
            ProductBranchCreationIntent::named("main").unwrap(),
            relational_basis,
            signal_basis,
            correspondence,
        );
        let world = RuntimeWorldOwner::builder()
            .with_execution_policy(worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    None,
                    Some(8_000_000),
                ),
            ))
            .with_bridge_correspondence(bridge_port)
            .with_relational_services(records.runtime.owner_component_services())
            .with_signal_services(services)
            .with_signal_definition_publication(signal_definition_publication)
            .with_budgets(limits)
            .with_clock(RuntimeWorldClock::from_source(
                crate::court::ordinary_context::budgets::CourtClock,
            ))
            .build()
            .expect("court declaration: World owner");
        Self {
            world,
            records,
            signal,
            bridge,
            installed,
            nodes,
            initial,
        }
    }
    pub(crate) fn bootstrap(&self) -> ProductBranchObservation {
        match self
            .world
            .lifecycle_port()
            .bootstrap_root(self.initial.clone())
            .expect("World owner available")
        {
            RuntimeWorldBootstrapOutcome::Performed(root) => root.product_branch().clone(),
            other => panic!("World bootstrap: {other:?}"),
        }
    }
    pub(crate) fn observe(&self, prior: &ProductBranchObservation) -> ProductBranchObservation {
        self.world
            .observation_port()
            .observe_product_branch(prior.branch_identity())
            .expect("observation: product head")
    }
    pub(crate) fn finish(mut self) {
        let inspection = self.world.inspection_port();
        let recovery = inspection.recovery_snapshot().unwrap();
        assert_eq!(
            (
                recovery.reserved(),
                recovery.abandoned(),
                recovery.updating()
            ),
            (0, 0, 0),
            "teardown: no pending attempts"
        );
        let report = self
            .world
            .lifecycle_port()
            .close()
            .expect("teardown: close World");
        assert_eq!(
            report.outstanding_observations(),
            0,
            "teardown: observations released"
        );
        assert!(
            report.retained_records().is_empty(),
            "teardown: no unexplained partials"
        );
        let relational_lifecycle = self
            .records
            .runtime
            .owner_component_services()
            .lifecycle_port();
        let signal_lifecycle = self
            .signal
            .owner_component_services()
            .unwrap()
            .lifecycle_port();
        let Self {
            world,
            records,
            signal,
            bridge,
            installed,
            initial,
            ..
        } = self;
        drop((world, bridge, installed, initial));
        let pins = records.runtime.branch_basis_cost_counters();
        assert_eq!(
            pins.external_retention_acquires, pins.external_retention_releases,
            "teardown: all external Relational leases released"
        );
        drop((records, signal));
        assert_eq!(
            relational_lifecycle.owner_lifecycle_observation(),
            worth_relational::facade::branch::RelationalOwnerLifecycleObservation::Closed
        );
        assert!(matches!(
            signal_lifecycle.owner_lifecycle_observation(),
            worth_signal::facade::branch::SignalOwnerLifecycleObservation::Closed
        ));
    }
}
