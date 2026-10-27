//! Concrete Query proof, observation and denial handoff; never a component authority.
use super::*;
use crate::domain_computation::primary_graph::application_query::read_execution::relational_read_denial;
use crate::domain_computation::primary_graph::{
    root_path_selection::{RootPathReadDenial as Denial, RootPathReadObservation as Observation},
    source_facts::SourceFactReadDenial,
};
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_query_installation::facade::WorthQueryInstalledRootPathGuard;
use worth_relational::facade::runtime::{
    BoundedFrontierFieldEqualityTruthRead, PinnedStructuralWalkDenialKind,
};
type Error = WorthQueryApplicationReadExecutionDenial;
pub(super) fn read_guard(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    guard: &WorthQueryInstalledRootPathGuard,
    candidates: &BTreeSet<EntityId>,
    prepare: &mut dyn FnMut() -> Result<(), Denial<Error>>,
) -> Result<BoundedFrontierFieldEqualityTruthRead, Denial<Error>> {
    let field = guard.field().as_str();
    let layout = graph
        .equality_field(guard.entity(), guard.aspect().as_str(), field)
        .ok_or_else(|| Denial::traversal(field))?;
    let computation = plan
        .governance
        .admit_internal_projection(
            (guard.entity(), guard.aspect().as_str(), field),
            guard.field(),
            ApplicationQueryObservableInfluence::RowPresence,
        )
        .ok_or_else(|| Denial::traversal(field))?;
    if !computation.admits_locator(&layout.locator) {
        return Err(Denial::traversal(field));
    }
    prepare()?;
    read_governed_root_guard(
        execution,
        runtime,
        computation,
        candidates,
        layout.entity_kind,
        &layout.locator,
        guard.expected(),
    )
    .map_err(|stop| Denial::frontier(field, stop))
}
pub(super) fn observe(
    work: &mut RootSelectionWork<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    event: Observation<'_>,
) -> Result<(), Error> {
    match event {
        Observation::SourceRead { subject } => work.charge_source_observation(subject),
        Observation::SourceCopy { units, subject } => work.charge_source_copy(units, subject),
        Observation::Adjacency {
            lists,
            records,
            endpoints,
            subject,
        } => work.charge(lists, records, endpoints, subject),
        Observation::Predicate {
            records,
            matches,
            fields,
            subject,
        } => work.charge_predicate(records, matches, fields, subject),
        Observation::Walk {
            reads,
            units,
            subject,
        } => work.charge_walk(reads, units, subject),
        Observation::Step {
            path_index,
            step_index,
            traversal,
            direction,
            reads,
            subject,
        } => work.record_step(
            execution, path_index, step_index, traversal, direction, reads, subject,
        ),
    }
}
pub(super) fn denial(error: Denial<Error>) -> Error {
    match error {
        Denial::Adapter(error) => error,
        Denial::Native { subject, cause } => relational_read_denial(cause, &subject),
        Denial::Source {
            subject,
            cause: SourceFactReadDenial::Execution(cause),
        } => relational_read_denial(cause, &subject),
        Denial::Source {
            subject,
            cause: SourceFactReadDenial::Unavailable,
        } => read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
            &subject,
        ),
        Denial::Source {
            subject,
            cause: SourceFactReadDenial::WorkLimitExceeded,
        }
        | Denial::FrontierLimit { subject } => work_limit_denial(&subject),
        Denial::Walk { subject, kind } => match kind {
            PinnedStructuralWalkDenialKind::Read(cause) => relational_read_denial(cause, &subject),
            PinnedStructuralWalkDenialKind::WorkLimitExceeded
            | PinnedStructuralWalkDenialKind::DistanceOverflow => work_limit_denial(&subject),
            _ => traversal_denial(&subject),
        },
        Denial::Traversal { subject } => traversal_denial(&subject),
    }
}
