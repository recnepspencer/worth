use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdvanceDenial, SignalBranchAdvanceOutcome,
    SignalOwnerCancellationSource, SignalOwnerServicePorts,
};
use worth_signal::facade::history::RuntimeBranch;
use worth_signal::facade::{
    Aspect, AspectVersion, DependencyEdge, EvaluationContext, NodeEvaluationResult, NodeId,
    SignalError, SignalGraph, SignalRuntime,
};

pub(super) const STORM_SEVERITY: Aspect = Aspect::new(0);
pub(super) const BERTH_AVAILABILITY: Aspect = Aspect::new(1);
pub(super) const MANIFEST_CLEARANCE: Aspect = Aspect::new(2);
pub(super) const VOYAGE_DISPATCHABILITY: Aspect = Aspect::new(3);
pub(super) const MEDICAL_CARGO_RELEASE: Aspect = Aspect::new(4);
pub(super) const INSPECTION_REQUIRED: Aspect = Aspect::new(5);

const DISPATCH_STORM_LIMIT: u64 = 5;
const MEDICAL_STORM_LIMIT: u64 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CargoContext {
    pub(super) storm_severity: u64,
    pub(super) berth_availability: u64,
    pub(super) manifest_clearance: u64,
}

impl CargoContext {
    pub(super) const fn baseline() -> Self {
        Self {
            storm_severity: 2,
            berth_availability: 3,
            manifest_clearance: 1,
        }
    }

    pub(super) const fn storm_front() -> Self {
        Self {
            storm_severity: 9,
            ..Self::baseline()
        }
    }

    pub(super) const fn berth_maintenance() -> Self {
        Self {
            berth_availability: 0,
            ..Self::baseline()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CargoNodes {
    pub(super) storm: NodeId,
    pub(super) berth: NodeId,
    pub(super) manifest: NodeId,
    pub(super) dispatchability: NodeId,
    pub(super) medical_release: NodeId,
    pub(super) inspection: NodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CargoOutputs {
    pub(super) voyage_dispatchable: bool,
    pub(super) medical_cargo_released: bool,
    pub(super) inspection_required: bool,
}

impl CargoOutputs {
    pub(super) const fn baseline() -> Self {
        Self {
            voyage_dispatchable: true,
            medical_cargo_released: true,
            inspection_required: false,
        }
    }

    pub(super) const fn storm_front() -> Self {
        Self {
            voyage_dispatchable: false,
            medical_cargo_released: false,
            inspection_required: true,
        }
    }

    pub(super) const fn berth_maintenance() -> Self {
        Self {
            voyage_dispatchable: false,
            medical_cargo_released: true,
            inspection_required: false,
        }
    }

    fn from_versions(versions: &[AspectVersion]) -> Self {
        assert_eq!(
            versions.len(),
            3,
            "the cargo court reads its three declared derived outputs"
        );
        Self {
            voyage_dispatchable: versions[0].get(VOYAGE_DISPATCHABILITY) == 1,
            medical_cargo_released: versions[1].get(MEDICAL_CARGO_RELEASE) == 1,
            inspection_required: versions[2].get(INSPECTION_REQUIRED) == 1,
        }
    }
}

pub(super) type CargoRuntime = SignalRuntime<(), (), (), CargoContext, ()>;
pub(super) type CargoServices = SignalOwnerServicePorts<(), (), (), CargoContext, ()>;

pub(super) struct CargoRoutingWorld {
    pub(super) runtime: Option<CargoRuntime>,
    pub(super) services: CargoServices,
    pub(super) nodes: CargoNodes,
    pub(super) main_branch: RuntimeBranch,
    pub(super) main_basis: AdmittedSignalBranchBasis,
}

impl CargoRoutingWorld {
    pub(super) fn new() -> Self {
        let (graph, nodes) = cargo_graph();
        let mut runtime = SignalRuntime::build_for::<CargoContext>(graph);
        let main_branch = runtime.current_branch();
        let main_basis = runtime
            .observe_signal_branch_basis(main_branch.clone())
            .expect("the production owner admits its bootstrap branch");
        let services = runtime
            .owner_component_services()
            .expect("the public owner bundle seals the valid cargo runtime");
        Self {
            runtime: Some(runtime),
            services,
            nodes,
            main_branch,
            main_basis,
        }
    }

    pub(super) fn establish_baseline(&mut self) -> CargoOutputs {
        let basis = self.main_basis.clone();
        let mut context = CargoContext::baseline();
        let (advance, outputs) = self
            .advance(&basis, &mut context, self.nodes.storm, STORM_SEVERITY)
            .expect("the baseline transaction performs through the mutation port");
        self.main_basis = advance.into_basis();
        outputs
    }

    pub(super) fn fork(
        &self,
        source: &AdmittedSignalBranchBasis,
        name: &str,
    ) -> (RuntimeBranch, AdmittedSignalBranchBasis) {
        let identity = worth_signal::facade::branch::validate_signal_branch_name(name)
            .expect("the semantic branch name is valid");
        self.services
            .mutation_port()
            .fork_exact(
                identity,
                source,
                &SignalOwnerCancellationSource::new().token(),
            )
            .expect("the public mutation port forks an owner-issued child")
            .into_parts()
    }

    pub(super) fn reference(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> worth_signal::facade::branch::ManagedSignalBranchReference {
        self.services
            .basis_port()
            .issue_managed_branch_reference(basis)
            .expect("a live owner basis issues a managed branch reference")
    }

    pub(super) fn observe(
        &self,
        reference: &worth_signal::facade::branch::ManagedSignalBranchReference,
    ) -> Result<
        AdmittedSignalBranchBasis,
        worth_signal::facade::branch::SignalBranchBasisObservationDenial,
    > {
        self.services.basis_port().observe_current(reference)
    }

    pub(super) fn advance(
        &self,
        expected: &AdmittedSignalBranchBasis,
        context: &mut CargoContext,
        changed_node: NodeId,
        changed_aspect: Aspect,
    ) -> Result<(SignalBranchAdvanceOutcome, CargoOutputs), SignalBranchAdvanceDenial> {
        let nodes = self.nodes;
        let evaluator = cargo_evaluator(nodes);
        let mut outputs = None;
        let outcome = self.services.mutation_port().advance_exact(
            expected,
            context,
            &SignalOwnerCancellationSource::new().token(),
            |transaction| {
                transaction.mark_changed(changed_node, changed_aspect)?;
                // A source value is materialized only when it is an explicit
                // transaction target. Reading derived nodes alone can reuse
                // their cached source versions after the context changes.
                transaction.read_many(&[nodes.storm, nodes.berth, nodes.manifest], &evaluator)?;
                let versions = transaction.read_many(
                    &[
                        nodes.dispatchability,
                        nodes.medical_release,
                        nodes.inspection,
                    ],
                    &evaluator,
                )?;
                outputs = Some(CargoOutputs::from_versions(&versions));
                Ok(())
            },
        )?;
        Ok((
            outcome,
            outputs.expect("the baseline evaluator observes all derived outputs"),
        ))
    }

    pub(super) fn close_owner(&mut self) {
        self.runtime.take();
    }
}

pub(super) fn populated_world() -> (CargoRoutingWorld, CargoOutputs) {
    let mut world = CargoRoutingWorld::new();
    let outputs = world.establish_baseline();
    (world, outputs)
}

fn cargo_graph() -> (SignalGraph, CargoNodes) {
    let mut graph = SignalGraph::new();
    let storm = graph.node().produces_aspects(STORM_SEVERITY).build();
    let berth = graph.node().produces_aspects(BERTH_AVAILABILITY).build();
    let manifest = graph.node().produces_aspects(MANIFEST_CLEARANCE).build();
    let dispatchability = graph
        .node()
        .on_demand()
        .produces_aspects(VOYAGE_DISPATCHABILITY)
        .build();
    let medical_release = graph
        .node()
        .on_demand()
        .produces_aspects(MEDICAL_CARGO_RELEASE)
        .build();
    let inspection = graph
        .node()
        .on_demand()
        .produces_aspects(INSPECTION_REQUIRED)
        .build();

    graph
        .set_dependencies(
            dispatchability,
            [
                DependencyEdge::new(storm, STORM_SEVERITY),
                DependencyEdge::new(berth, BERTH_AVAILABILITY),
                DependencyEdge::new(manifest, MANIFEST_CLEARANCE),
            ],
        )
        .expect("voyage dispatchability dependencies are valid");
    graph
        .set_dependencies(
            medical_release,
            [
                DependencyEdge::new(storm, STORM_SEVERITY),
                DependencyEdge::new(manifest, MANIFEST_CLEARANCE),
            ],
        )
        .expect("medical release dependencies are valid");
    graph
        .set_dependencies(
            inspection,
            [
                DependencyEdge::new(storm, STORM_SEVERITY),
                DependencyEdge::new(manifest, MANIFEST_CLEARANCE),
            ],
        )
        .expect("inspection dependencies are valid");

    (
        graph,
        CargoNodes {
            storm,
            berth,
            manifest,
            dispatchability,
            medical_release,
            inspection,
        },
    )
}

fn cargo_evaluator(
    nodes: CargoNodes,
) -> impl for<'ctx> Fn(
    &mut EvaluationContext<'ctx, CargoContext>,
) -> Result<NodeEvaluationResult, SignalError>
       + Copy
       + Send
       + Sync {
    move |view| evaluate_cargo_node(view, nodes)
}

fn evaluate_cargo_node(
    view: &mut EvaluationContext<'_, CargoContext>,
    nodes: CargoNodes,
) -> Result<NodeEvaluationResult, SignalError> {
    if view.node() == nodes.storm {
        return Ok(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(STORM_SEVERITY, view.domain().storm_severity)]),
        ));
    }
    if view.node() == nodes.berth {
        return Ok(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(BERTH_AVAILABILITY, view.domain().berth_availability)]),
        ));
    }
    if view.node() == nodes.manifest {
        return Ok(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(MANIFEST_CLEARANCE, view.domain().manifest_clearance)]),
        ));
    }

    let storm = view.read(nodes.storm, STORM_SEVERITY)?;
    let manifest = view.read(nodes.manifest, MANIFEST_CLEARANCE)?;
    if view.node() == nodes.dispatchability {
        let berth = view.read(nodes.berth, BERTH_AVAILABILITY)?;
        let value = u64::from(storm <= DISPATCH_STORM_LIMIT && berth > 0 && manifest > 0);
        return Ok(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(VOYAGE_DISPATCHABILITY, value)]),
        ));
    }
    if view.node() == nodes.medical_release {
        let value = u64::from(storm <= MEDICAL_STORM_LIMIT && manifest > 0);
        return Ok(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(MEDICAL_CARGO_RELEASE, value)]),
        ));
    }
    if view.node() == nodes.inspection {
        let value = u64::from(storm > DISPATCH_STORM_LIMIT || manifest == 0);
        return Ok(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(INSPECTION_REQUIRED, value)]),
        ));
    }

    Err(SignalError::invalid_input("unknown cargo court node"))
}
