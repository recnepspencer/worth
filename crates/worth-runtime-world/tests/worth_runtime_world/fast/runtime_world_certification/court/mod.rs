#[path = "bootstrap.rs"]
pub(crate) mod bootstrap;

#[path = "branches.rs"]
pub(crate) mod branches;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::court::bridge_delivery;

#[path = "../../../fixtures/runtime_world_certification/court/budgets.rs"]
pub(crate) mod budgets;

#[path = "../../../fixtures/runtime_world_certification/court/correspondence.rs"]
pub(crate) mod correspondence;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::court::cost;

#[path = "exact_history.rs"]
pub(crate) mod exact_history;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::court::model;

#[path = "../../../fixtures/runtime_world_certification/court/observation.rs"]
pub(crate) mod observation;

#[path = "operation_control/mod.rs"]
pub(crate) mod operation_control;

#[path = "../../../fixtures/runtime_world_certification/court/operations.rs"]
pub(crate) mod operations;

#[path = "../../../fixtures/runtime_world_certification/court/oracle.rs"]
pub(crate) mod oracle;

#[path = "outcomes.rs"]
pub(crate) mod outcomes;

#[path = "record_history/mod.rs"]
pub(crate) mod record_history;

#[path = "recovery.rs"]
pub(crate) mod recovery;

#[path = "../../../fixtures/runtime_world_certification/court/relational.rs"]
pub(crate) mod relational;

#[path = "retention.rs"]
pub(crate) mod retention;

#[path = "../../../fixtures/runtime_world_certification/court/routing.rs"]
pub(crate) mod routing;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::court::sequential;

pub(crate) use relational::CargoRecords;
pub(crate) use routing::{
    CargoEvent, CargoKey, CourtSignal, RouteDefinition, RouteGraph, RoutingContext,
};
pub(crate) use worth_runtime_bridge::facade::{
    BridgeInstalledSemanticCorrespondence, RuntimeBridge,
};
pub(crate) use worth_runtime_world::facade::*;
pub(crate) use worth_signal::facade::SignalGraph;

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
        Self::compile_config(include_steel, budgets::court())
    }
    pub(crate) fn compile_config(include_steel: bool, limits: RuntimeWorldBudgets) -> Self {
        let records = CargoRecords::install(include_steel);
        let mut graph = SignalGraph::new();
        let nodes = RouteGraph::install(&mut graph);
        let (bridge, installed, _) = correspondence::install(&records, &mut graph, nodes);
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
            .with_clock(RuntimeWorldClock::from_source(budgets::CourtClock))
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
