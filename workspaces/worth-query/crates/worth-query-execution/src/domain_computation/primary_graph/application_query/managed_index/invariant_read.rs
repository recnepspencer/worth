//! A candidate read reborrows the actual observation and original request each time.
mod projection;
use crate::domain_computation::primary_graph::vocabulary::application_query::invariant_read::ManagedSourceValue;
use crate::domain_computation::primary_graph::vocabulary::application_query::invariant_read::ReadPosture;
use crate::domain_computation::primary_graph::vocabulary::application_query::invariant_read::WorthQueryManagedIndexInvariantDenial;
use worth_query_declaration::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding},
    application_schema::{ApplicationScalarValueBinding, ApplicationSchema},
};
use worth_relational::facade::runtime::{
    ManagedIndexSourcePreparationDenial as NativeDenial, ManagedIndexSourceRead,
};
use {
    super::invocation::decoded_provider, super::source::projection::project_row,
    super::source::service::ManagedSourceRoot,
    crate::domain_computation::primary_graph::vocabulary::application_query::provider::WorthQueryManagedIndexProbeValue,
};
use {
    crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection,
    crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjectionRow,
};

pub(in crate::domain_computation::primary_graph) fn query<S, I>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &ManagedIndexSourceRead<'_>,
    program: &worth_relational::facade::runtime::ManagedIndexSourceProgram,
    posture: ReadPosture,
    match_posture: worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture,
    probe: &WorthQueryManagedIndexProbeValue<S, I>,
) -> Result<Vec<ManagedSourceValue<S, I>>, WorthQueryManagedIndexInvariantDenial>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    ManagedSourceValue<S, I>:
        WorthQueryApplicationProjection<S, <I::SourceBinding as ApplicationQueryBinding<S>>::Query>,
{
    use WorthQueryManagedIndexInvariantDenial as Denial;
    let payload = match posture {
        ReadPosture::Committed => source.committed_root()?,
        ReadPosture::Proposed => source.proposed_root()?,
    };
    let root = payload
        .downcast_ref::<ManagedSourceRoot<S, I>>()
        .ok_or(NativeDenial::ForeignSource)?;
    let rows = match posture {
        ReadPosture::Committed => source.committed()?,
        ReadPosture::Proposed => source.proposed(),
    };
    let budget = source.read_budget();
    let probe = I::ProbeBinding::encode(probe).map_err(Denial::Probe)?;
    let frame = crate::domain_computation::primary_graph::access_strategy::coverage::CoverageFrame::invariant(
        root.rows.view(&root.bindings, Some(&budget)), &root.nodes, root, &probe, source, posture, match_posture,
    );
    let selection = root
        .adapter
        .candidates(execution, frame)
        .map_err(Denial::Provider)?;
    let selected = super::invocation::controlled(execution, |control| {
        let control = control.under_source(&budget);
        selection.receive(frame, root.adapter.coverage_authority(), true, &control)
    })
    .map_err(Denial::Provider)?;
    projection::read::<S, I>(execution, source, program, root, &rows, selected)
}
