//! Component guard authority is the exact installed path closure, not Query governance.
use super::*;
use crate::domain_computation::primary_graph::{
    root_path_selection::{
        self, RootPathOrientation, RootPathReadBasis, RootPathReadDenial as PathDenial,
        RootPathReadObservation as Observation, RootPathReadPorts, RootPathSelection,
        RootPathSelectionRequest,
    },
    source_facts::SourceFactReadDenial,
};
use worth_query_installation::facade::WorthQueryInstalledRootPathGuard;
use worth_relational::facade::{
    runtime::{BoundedFrontierFieldEqualityTruthRead, PinnedStructuralWalkDenialKind},
    transactions::RelationalExecutionDenialCause as Cause,
};
impl<Schema: ApplicationSchema, Operation: ApplicationOperationMarkerIdentity<Schema>>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(super) fn select_component_paths(
        &mut self,
        scope: &WorthQueryOperationComponentReadScope,
        start: EntityId,
        orientation: RootPathOrientation,
        capture_set: bool,
        pin: &RelationalPinnedRead,
        subject: &str,
        allocation: &Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
    ) -> Result<RootPathSelection, Denial> {
        let execution = *self.reader.execution;
        let graph = self.reader.layout;
        let projection = pin
            .project_snapshot(self.reader.snapshot)
            .ok_or_else(|| Denial::new(Kind::EndpointUnavailable, subject))?;
        let mut reads = WorthQueryInvariantProjectionWork::default();
        let checkpoint = |_: &str| execution.consult().map_err(Cause::from);
        let mut guard =
            |guard: &WorthQueryInstalledRootPathGuard,
             candidates: &BTreeSet<EntityId>,
             prepare: &mut dyn FnMut() -> Result<(), PathDenial<Cause>>| {
                read_guard(&execution, pin, graph, scope, guard, candidates, prepare)
            };
        let mut observe = |event: Observation<'_>| {
            record_reads(&mut reads, event);
            Ok(())
        };
        let result = root_path_selection::select(
            &RootPathReadBasis {
                execution: &execution,
                runtime: pin,
                graph,
                projection,
            },
            RootPathSelectionRequest {
                allocation: Some(allocation),
                paths: scope.paths(),
                start,
                orientation,
                capture_set,
                subject,
            },
            &mut RootPathReadPorts {
                checkpoint: &checkpoint,
                read_guard_with_sources: &mut guard,
                observation: &mut observe,
            },
        );
        self.reader.work = self.reader.work.combined(reads);
        result.map_err(|denial| self.component_path_denial(denial, subject))
    }
    pub(super) fn component_source_denial(
        &mut self,
        cause: SourceFactReadDenial,
        subject: &str,
    ) -> Denial {
        match cause {
            SourceFactReadDenial::Execution(cause) => self.reader.walk_stop(cause, subject),
            SourceFactReadDenial::Unavailable => Denial::new(Kind::EndpointUnavailable, subject),
            SourceFactReadDenial::WorkLimitExceeded => {
                Denial::new(Kind::WorkBudgetExceeded, subject)
            }
        }
    }
    fn component_path_denial(&mut self, denial: PathDenial<Cause>, subject: &str) -> Denial {
        match denial {
            PathDenial::Adapter(cause) | PathDenial::Native { cause, .. } => {
                self.reader.walk_stop(cause, subject)
            }
            PathDenial::Source { cause, .. } => self.component_source_denial(cause, subject),
            PathDenial::Walk {
                kind: PinnedStructuralWalkDenialKind::Read(cause),
                ..
            } => self.reader.walk_stop(cause, subject),
            PathDenial::Walk {
                kind:
                    PinnedStructuralWalkDenialKind::WorkLimitExceeded
                    | PinnedStructuralWalkDenialKind::DistanceOverflow,
                ..
            }
            | PathDenial::FrontierLimit { .. } => Denial::new(Kind::WorkBudgetExceeded, subject),
            PathDenial::Walk { .. } | PathDenial::Traversal { .. } => {
                Denial::new(Kind::EndpointUnavailable, subject)
            }
        }
    }
}
fn read_guard(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    pin: &RelationalPinnedRead,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    scope: &WorthQueryOperationComponentReadScope,
    guard: &WorthQueryInstalledRootPathGuard,
    candidates: &BTreeSet<EntityId>,
    prepare: &mut dyn FnMut() -> Result<(), PathDenial<Cause>>,
) -> Result<BoundedFrontierFieldEqualityTruthRead, PathDenial<Cause>> {
    let field = guard.field().as_str();
    // The kernel passes references from this exact installed closure. Descriptive
    // equality, an atomic field grant, or a guard from another closure cannot admit it.
    if !scope
        .paths()
        .iter()
        .flat_map(|path| path.guards())
        .any(|admitted| std::ptr::eq(admitted, guard))
    {
        return Err(PathDenial::traversal(field));
    }
    let layout = graph
        .equality_field(guard.entity(), guard.aspect().as_str(), field)
        .ok_or_else(|| PathDenial::traversal(field))?;
    prepare()?;
    pin.bounded_entity_field_equals_for_frontier(
        execution,
        candidates,
        layout.entity_kind,
        &layout.locator,
        guard.expected(),
        None,
    )
    .map_err(|stop| PathDenial::frontier(field, stop))
}
fn record_reads(reads: &mut WorthQueryInvariantProjectionWork, event: Observation<'_>) {
    match event {
        Observation::Adjacency {
            lists,
            records,
            endpoints,
            ..
        } => {
            reads.record_adjacencies(lists, records);
            reads.record_endpoints(endpoints);
        }
        Observation::Predicate { fields, .. } => reads.record_fields(fields),
        Observation::Walk { reads: native, .. } => {
            reads.record_adjacencies(native.adjacency_lists, native.relation_records);
            reads.record_endpoints(native.entity_records);
        }
        Observation::SourceRead { .. }
        | Observation::SourceCopy { .. }
        | Observation::Step { .. } => {}
    }
}
