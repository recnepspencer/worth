//! One physical traversal engine for installed Query and operation member paths.
//! Real read adapters own admission; this kernel cannot turn descriptors into permission.
use super::{source_facts::SourceFactsBuilder, WorthQueryPrimaryGraphLayout};
use std::collections::{BTreeMap, BTreeSet};
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
}
pub(in crate::domain_computation::primary_graph) struct RootPathSelection {
    pub(in crate::domain_computation::primary_graph) roots: Frontier,
    pub(in crate::domain_computation::primary_graph) set_source: Option<SourceFactsBuilder>,
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
    let mut roots = BTreeMap::new();
    let mut set_source = request.capture_set.then(SourceFactsBuilder::default);
    if let Some(source) = &mut set_source {
        source.observe_entity(request.start);
    }
    for (path_index, path) in request.paths.iter().enumerate() {
        ports.check(request.subject)?;
        let mut terminal =
            traverse_path(basis, &request, path, path_index, ports, &mut set_source)?;
        match request.orientation {
            RootPathOrientation::ForwardCensus => {
                for (root, source) in terminal {
                    roots.entry(root).or_insert(source);
                }
            }
            RootPathOrientation::ReverseWitness { required_root } => {
                if let Some(source) = terminal.remove(&required_root) {
                    roots.insert(required_root, source);
                    break;
                }
            }
        }
    }
    if let Some(source) = &mut set_source {
        for witness in roots.values() {
            for entity in witness.entities() {
                if source.observe_entity(entity) {
                    ports.notify(RootPathReadObservation::SourceCopy {
                        units: 1,
                        subject: request.subject,
                    })?;
                }
            }
        }
    }
    Ok(RootPathSelection { roots, set_source })
}
fn traverse_path<E>(
    basis: &RootPathReadBasis<'_, '_, '_>,
    request: &RootPathSelectionRequest<'_>,
    path: &WorthQueryInstalledRootPath,
    path_index: usize,
    ports: &mut RootPathReadPorts<'_, E>,
    set_source: &mut Option<SourceFactsBuilder>,
) -> Result<Frontier, RootPathReadDenial<E>> {
    let mut seed = SourceFactsBuilder::default();
    seed.observe_entity(request.start);
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
        set_source,
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
            closure::advance(basis, layout, located, &frontier, ports, set_source)?
        } else {
            step::advance(basis, layout, located, &mut frontier, ports, set_source)?
        };
        guards::apply(
            basis,
            path,
            if reverse { step_index } else { step_index + 1 },
            &mut frontier,
            ports,
            set_source,
        )?;
    }
    Ok(frontier)
}
