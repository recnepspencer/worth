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
mod closure;
mod denial;
mod guards;
mod ports;
mod step;
mod storage;
pub(in crate::domain_computation::primary_graph) use denial::RootPathReadDenial;
pub(in crate::domain_computation::primary_graph) use ports::{
    RootPathReadObservation, RootPathReadPorts,
};
type Frontier = BTreeMap<EntityId, SourceFactsBuilder>;

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum RootPathOrientation {
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
pub(in crate::domain_computation::primary_graph) struct RootPathSelectionRequest<'a> {
    pub(in crate::domain_computation::primary_graph) paths: &'a [WorthQueryInstalledRootPath],
    pub(in crate::domain_computation::primary_graph) start: EntityId,
    pub(in crate::domain_computation::primary_graph) orientation: RootPathOrientation,
    pub(in crate::domain_computation::primary_graph) capture_set: bool,
    pub(in crate::domain_computation::primary_graph) subject: &'a str,
    pub(in crate::domain_computation::primary_graph) allocation:
        Option<&'a Arc<SourceFactsAllocation>>,
}
pub(in crate::domain_computation::primary_graph) struct RootPathSelection {
    pub(in crate::domain_computation::primary_graph) roots: BTreeMap<EntityId, SourceFacts>,
    pub(in crate::domain_computation::primary_graph) set_source: Option<SourceFacts>,
    pub(in crate::domain_computation::primary_graph) allocation: Option<Arc<SourceFactsAllocation>>,
}
struct RootPathSelectionBuilder {
    pub(in crate::domain_computation::primary_graph) roots: Frontier,
    pub(in crate::domain_computation::primary_graph) set_source: Option<SourceFactsBuilder>,
    pub(in crate::domain_computation::primary_graph) allocation: Option<Arc<SourceFactsAllocation>>,
}
pub(in crate::domain_computation::primary_graph) struct RootPathReadBasis<'a, 'execution, 'request>
{
    pub(in crate::domain_computation::primary_graph) execution:
        &'a ExecutionRequest<'execution, 'request>,
    pub(in crate::domain_computation::primary_graph) runtime: &'a RelationalPinnedRead,
    pub(in crate::domain_computation::primary_graph) graph: &'a WorthQueryPrimaryGraphLayout,
    pub(in crate::domain_computation::primary_graph) projection: VisibilityProjectionView,
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
            traversal: self.step.traversal(),
            direction: self.direction,
            reads,
            subject: self.step.relation(),
        }
    }
}

/// Forward visits the complete pre-window union. Reverse returns a full accepted
/// witness to the exact required root, or the negative evidence of every path.
pub(in crate::domain_computation::primary_graph) fn select<E>(
    basis: &RootPathReadBasis<'_, '_, '_>,
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
fn traverse_path<E>(
    basis: &RootPathReadBasis<'_, '_, '_>,
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
