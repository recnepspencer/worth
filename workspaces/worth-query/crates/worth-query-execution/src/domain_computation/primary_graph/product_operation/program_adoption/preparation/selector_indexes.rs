//! Candidate-owned maintenance of installed authorization selector generations.
use super::{
    WorthQueryBranchAdoptionPreparationDenial as Denial, WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;

pub(super) fn prepare<Schema: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    candidate: &mut PreparedRelationalCommitCandidate,
) -> Result<(), Denial> {
    let application = selected.application();
    let graph = &application.primary_provider.graph;
    let mut indexes = std::collections::BTreeSet::new();
    indexes.extend(
        graph
            .layout
            .principal_bindings()
            .map(|(_, binding)| binding.selector_index()),
    );
    // Installed capability operations remain in the schema even when the current
    // program removes ordinary action admission. Their read/elevation footprint
    // therefore preserves exact performed-effect recovery dependencies.
    indexes.extend(graph.layout.authorization_selector_index_ids());
    // Grant selection consumes these bounded joins at the same successor basis.
    // Their native patch evaluator shares the authentic predecessor and payer.
    indexes.extend(graph.layout.capability_grant_join_index_ids());
    // The next adoption reads this exact lineage population. Carry its
    // selected generation with the candidate, including an empty population.
    indexes.insert(graph.layout.workflow().lineage.identity_index_id);
    let indexes = indexes.into_iter().collect::<Vec<_>>();
    graph.with_runtime_mut(execution, |runtime| {
        let before = super::super::super::super::exact_basis_access::open_exact_basis_snapshot(
            runtime, selected.product().relational_basis(),
        ).map_err(Denial::SelectorSnapshot)?;
        let result = runtime.index_authority().prepare_incremental_indexes_for_candidate(
            candidate, &indexes, &before,
            super::super::super::super::index_maintenance_budget::ordinary_index_maintenance_budget(),
            execution,
        ).map(|_| ()).map_err(Denial::SelectorMaintenance);
        crate::relational_snapshot_release::release_query_snapshot(runtime, &before);
        result
    })
}
