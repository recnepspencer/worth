//! Candidate-owned successor generations for installed reads and authorization.
use super::WorthQuerySelectedProductOperation;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;

#[derive(Debug)]
pub(super) enum CandidateSelectorMaintenanceDenial {
    Maintenance(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial),
}

pub(super) fn prepare<S: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    candidate: &mut PreparedRelationalCommitCandidate,
) -> Result<(), CandidateSelectorMaintenanceDenial> {
    let graph = &selected.application().runtime.primary_provider.graph;
    graph.with_runtime_mut(execution, |runtime| {
        runtime
            .index_authority()
            .prepare_routed_indexes_for_candidate(
                candidate,
                super::super::index_maintenance_budget::ordinary_index_maintenance_budget(),
                execution,
            )
            .map(|_| ())
            .map_err(CandidateSelectorMaintenanceDenial::Maintenance)
    })
}
