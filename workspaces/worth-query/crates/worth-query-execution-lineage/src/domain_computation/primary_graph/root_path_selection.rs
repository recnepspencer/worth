//! One physical traversal engine for installed Query and operation member paths.
//! Real read adapters own admission; this kernel cannot turn descriptors into permission.
use super::{
    source_facts::{SourceFacts, SourceFactsAllocation, SourceFactsBuilder},
    WorthQueryPrimaryGraphLayout,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryRootPathTraversal, RelationDirection,
};
use worth_query_installation::facade::{
    WorthQueryInstalledRootPath, WorthQueryInstalledRootPathStep,
};
use worth_relational::facade::{
    identity::EntityId,
    runtime::{RelationalPinnedRead, VisibilityProjectionView},
};
pub mod closure;
pub mod denial;
pub mod derived;
pub mod guards;
pub mod ports;
pub mod step;
pub mod storage;
pub use denial::RootPathReadDenial;
pub use ports::{RootPathReadObservation, RootPathReadPorts};
type Frontier = BTreeMap<EntityId, SourceFactsBuilder>;

#[derive(Clone, Copy)]
pub enum RootPathOrientation {
    ForwardCensus,
    ReverseWitness { required_root: EntityId },
}
impl RootPathOrientation {
    fn direction(&self, direction: RelationDirection) -> RelationDirection {
        match (self, direction) {
            (Self::ForwardCensus, direction) => direction,
            (Self::ReverseWitness { .. }, RelationDirection::Forward) => RelationDirection::Reverse,
            (Self::ReverseWitness { .. }, RelationDirection::Reverse) => RelationDirection::Forward,
        }
    }
}
pub struct RootPathSelectionRequest<'a> {
    pub paths: &'a [WorthQueryInstalledRootPath],
    pub start: EntityId,
    pub orientation: RootPathOrientation,
    pub capture_set: bool,
    pub subject: &'a str,
    pub allocation: Option<&'a Arc<SourceFactsAllocation>>,
}
pub struct RootPathSelection {
    pub roots: BTreeMap<EntityId, SourceFacts>,
    pub set_source: Option<SourceFacts>,
    pub allocation: Option<Arc<SourceFactsAllocation>>,
}
struct RootPathSelectionBuilder {
    pub roots: Frontier,
    pub set_source: Option<SourceFactsBuilder>,
    pub allocation: Option<Arc<SourceFactsAllocation>>,
}
pub struct RootPathReadBasis<
    'a,
    'execution,
    'request,
    Phase = worth_relational::facade::snapshots::SnapshotHandle,
> {
    pub execution: &'a ExecutionRequest<'execution, 'request>,
    pub runtime: &'a RelationalPinnedRead<Phase>,
    pub origin: super::projection_read_origin::ProjectionReadOriginRef<'a>,
    pub graph: &'a WorthQueryPrimaryGraphLayout,
    pub projection: VisibilityProjectionView,
}
#[derive(Clone, Copy)]
struct PathStep<'a> {
    path_index: usize,
    step_index: usize,
    step: &'a WorthQueryInstalledRootPathStep,
    direction: RelationDirection,
}
impl<'a> PathStep<'a> {
    fn completed(
        self,
        reads: super::read_observation::WorthQueryInvariantProjectionWork,
    ) -> RootPathReadObservation<'a> {
        RootPathReadObservation::Step {
            path_index: self.path_index,
            step_index: self.step_index,
            source_kind: self.step.source_kind(),
            traversal: self.step.traversal(),
            direction: self.direction,
            reads,
            subject: self.step.relation(),
        }
    }
}

/// Forward visits the complete pre-window union. Reverse returns a full accepted
/// witness to the exact required root, or the negative evidence of every path.
pub fn select<E, Phase>(
    basis: &RootPathReadBasis<'_, '_, '_, Phase>,
    request: RootPathSelectionRequest<'_>,
    ports: &mut RootPathReadPorts<'_, E>,
) -> Result<RootPathSelection, RootPathReadDenial<E>> {
    let mut selected = RootPathSelectionBuilder {
        roots: BTreeMap::new(),
        set_source: request.capture_set.then(SourceFactsBuilder::default),
        allocation: request.allocation.cloned(),
    };
    if let Some(source) = &mut selected.set_source {
        if let Some(allocation) = &selected.allocation {
            source
                .protect(basis.execution, allocation)
                .map_err(|cause| RootPathReadDenial::source(request.subject, cause))?;
        }
        source
            .observe_entity(basis.execution, request.start)
            .map_err(|cause| RootPathReadDenial::native(request.subject, cause))?;
    }
    for (path_index, path) in request.paths.iter().enumerate() {
        ports.check(request.subject)?;
        let mut terminal = traverse_path(basis, &request, path, path_index, ports, &mut selected)?;
        match request.orientation {
            RootPathOrientation::ForwardCensus => {
                for (root, source) in terminal {
                    if !selected.roots.contains_key(&root) {
                        selected
                            .admit_frontier_entry(basis.execution, selected.roots.len())
                            .map_err(|cause| RootPathReadDenial::native(request.subject, cause))?;
                        selected.roots.insert(root, source);
                    }
                }
            }
            RootPathOrientation::ReverseWitness { required_root } => {
                if let Some(source) = terminal.remove(&required_root) {
                    selected
                        .admit_frontier_entry(basis.execution, selected.roots.len())
                        .map_err(|cause| RootPathReadDenial::native(request.subject, cause))?;
                    selected.roots.insert(required_root, source);
                    break;
                }
            }
        }
    }
    if let Some(source) = &mut selected.set_source {
        for witness in selected.roots.values() {
            for entity in witness.entities() {
                if source
                    .observe_entity(basis.execution, entity)
                    .map_err(|cause| RootPathReadDenial::native(request.subject, cause))?
                {
                    ports.notify(RootPathReadObservation::SourceCopy {
                        units: 1,
                        subject: request.subject,
                    })?;
                }
            }
        }
    }
    selected
        .finish(basis.execution)
        .map_err(|cause| RootPathReadDenial::native(request.subject, cause))
}
fn traverse_path<E, Phase>(
    basis: &RootPathReadBasis<'_, '_, '_, Phase>,
    request: &RootPathSelectionRequest<'_>,
    path: &WorthQueryInstalledRootPath,
    path_index: usize,
    ports: &mut RootPathReadPorts<'_, E>,
    selected: &mut RootPathSelectionBuilder,
) -> Result<Frontier, RootPathReadDenial<E>> {
    let mut seed = SourceFactsBuilder::default();
    if let Some(allocation) = &selected.allocation {
        seed.protect(basis.execution, allocation)
            .map_err(|cause| RootPathReadDenial::source(request.subject, cause))?;
    }
    seed.observe_entity(basis.execution, request.start)
        .map_err(|cause| RootPathReadDenial::native(request.subject, cause))?;
    selected
        .admit_frontier_entry(basis.execution, 0)
        .map_err(|cause| RootPathReadDenial::native(request.subject, cause))?;
    let mut frontier = BTreeMap::from([(request.start, seed)]);
    let count = path.steps().len();
    let reverse = matches!(
        request.orientation,
        RootPathOrientation::ReverseWitness { .. }
    );
    guards::apply(
        basis,
        path,
        if reverse { count } else { 0 },
        &mut frontier,
        ports,
        selected,
    )?;
    for offset in 0..count {
        let step_index = if reverse { count - 1 - offset } else { offset };
        let step = &path.steps()[step_index];
        let located = PathStep {
            path_index,
            step_index,
            step,
            direction: request.orientation.direction(step.direction()),
        };
        if frontier.is_empty() {
            ports.notify(located.completed(Default::default()))?;
            continue;
        }
        if step.source_kind() == worth_query_declaration::facade::application_query::ApplicationQueryRootPathSourceKind::Derived {
            if step.traversal() != ApplicationQueryRootPathTraversal::OneHop { return Err(RootPathReadDenial::traversal(step.relation())); }
            frontier = derived::advance(basis, located, &mut frontier, ports, selected)?;
        } else {
        let layout = basis
            .graph
            .relation(step.relation())
            .filter(|layout| {
                basis.graph.entity_kind(step.from()) == Some(layout.from)
                    && basis.graph.entity_kind(step.to()) == Some(layout.to)
            })
            .ok_or_else(|| RootPathReadDenial::traversal(step.relation()))?;
        frontier = if step.traversal() == ApplicationQueryRootPathTraversal::Closure {
            closure::advance(basis, layout, located, &mut frontier, ports, selected)?
        } else {
            step::advance(basis, layout, located, &mut frontier, ports, selected)?
        };
        }
        guards::apply(
            basis,
            path,
            if reverse { step_index } else { step_index + 1 },
            &mut frontier,
            ports,
            selected,
        )?;
    }
    Ok(frontier)
}
