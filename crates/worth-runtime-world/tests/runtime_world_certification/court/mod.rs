mod bootstrap;
mod branches;
mod bridge_delivery;
mod budgets;
mod correspondence;
mod cost;
mod model;
mod observation;
#[cfg(feature = "test-operation-control")]
mod operation_control;
mod operations;
mod oracle;
mod outcomes;
mod recovery;
mod relational;
mod retention;
mod routing;
mod sequential;

use relational::CargoRecords;
use routing::{CargoEvent, CargoKey, CourtSignal, RouteDefinition, RouteGraph, RoutingContext};
use worth_runtime_bridge::facade::{BridgeInstalledSemanticCorrespondence, RuntimeBridge};
use worth_runtime_world::facade::*;
use worth_signal::facade::SignalGraph;

pub struct CompositeSupplyChainCourt {
    pub world: RuntimeWorldOwner<RouteDefinition, CargoKey, CargoEvent, RoutingContext, u64>,
    pub records: CargoRecords,
    pub signal: CourtSignal,
    pub bridge: RuntimeBridge,
    pub installed: BridgeInstalledSemanticCorrespondence,
    pub nodes: RouteGraph,
    pub initial: RuntimeWorldBootstrapIntent,
}
impl CompositeSupplyChainCourt {
    pub fn compile() -> Self {
        Self::compile_with_steel(true)
    }
    pub fn compile_with_steel(include_steel: bool) -> Self {
        Self::compile_config(include_steel, budgets::court())
    }
    pub fn compile_config(include_steel: bool, limits: RuntimeWorldBudgets) -> Self {
        let records = CargoRecords::install(include_steel);
        let mut graph = SignalGraph::new();
        let nodes = RouteGraph::install(&mut graph);
        let (bridge, installed, _) = correspondence::install(&records, &mut graph, nodes);
        Self::from_installed_graph(records, graph, nodes, bridge, installed, limits)
    }
    pub(super) fn from_installed_graph(
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
    pub fn bootstrap(&self) -> ProductBranchObservation {
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
    pub fn observe(&self, prior: &ProductBranchObservation) -> ProductBranchObservation {
        self.world
            .observation_port()
            .observe_product_branch(prior.branch_identity())
            .expect("observation: product head")
    }
    pub fn finish(mut self) {
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
