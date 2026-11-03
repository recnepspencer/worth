//! Candidate-owned successor generations for installed reads and authorization.
use super::WorthQuerySelectedProductOperation;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;

#[derive(Debug)]
pub(super) enum CandidateSelectorMaintenanceDenial {
    Snapshot(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
    Maintenance(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial),
}

fn indexes<S: ApplicationSchema>(
    selected: &WorthQuerySelectedProductOperation<'_, S>,
) -> Vec<worth_relational::facade::indexes::DerivedIndexId> {
    let graph = &selected.application().runtime.primary_provider.graph;
    let mut indexes = std::collections::BTreeSet::new();
    indexes.extend(graph.primary_index_ids.iter().copied());
    indexes.extend(
        graph
            .layout
            .principal_bindings()
            .map(|(_, binding)| binding.selector_index()),
    );
    indexes.extend(graph.layout.authorization_selector_index_ids());
    indexes.extend(graph.layout.capability_grant_join_index_ids());
    indexes.insert(graph.layout.workflow().lineage.identity_index_id);
    indexes.insert(graph.layout.workflow().instance.identity_index_id);
    if let Some(reads) = graph.layout.workflow_reads() {
        indexes.extend([reads.instances, reads.transitions]);
    }
    indexes.into_iter().collect()
}

/// Publish the fork's own exact generations from its canonical author's seed.
/// The ordinary budget prohibits population reconstruction if that seed is absent.
fn seed<S: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    indexes: Vec<worth_relational::facade::indexes::DerivedIndexId>,
) -> Result<(), CandidateSelectorMaintenanceDenial> {
    let graph = &selected.application().runtime.primary_provider.graph;
    let basis = selected.product().relational_basis();
    let observation = basis.observation();
    let Some(source_commit_id) = observation.commit_id() else {
        return Ok(());
    };
    let request = worth_relational::facade::indexes::DerivedIndexBuildRequest {
        source_commit_id,
        branch_id: observation.identity().branch_id().clone(),
        index_ids: indexes,
    };
    graph.with_runtime(execution, |runtime| {
        runtime
            .index_authority()
            .refresh_for_basis(
                request,
                basis,
                None,
                super::super::index_maintenance_budget::ordinary_index_maintenance_budget(),
                execution,
            )
            .map(|_| ())
            .map_err(CandidateSelectorMaintenanceDenial::Maintenance)
    })
}

pub(super) fn prepare<S: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    candidate: &mut PreparedRelationalCommitCandidate,
) -> Result<(), CandidateSelectorMaintenanceDenial> {
    let graph = &selected.application().runtime.primary_provider.graph;
    let indexes = indexes(selected);
    seed(execution, selected, indexes.clone())?;
    graph.with_runtime_mut(execution, |runtime| {
        let before = super::super::exact_basis_access::open_exact_basis_snapshot(
            runtime,
            selected.product().relational_basis(),
        )
        .map_err(CandidateSelectorMaintenanceDenial::Snapshot)?;
        let result = runtime
            .index_authority()
            .prepare_incremental_indexes_for_candidate(
                candidate,
                &indexes,
                &before,
                super::super::index_maintenance_budget::ordinary_index_maintenance_budget(),
                execution,
            )
            .map(|_| ())
            .map_err(CandidateSelectorMaintenanceDenial::Maintenance);
        crate::relational_snapshot_release::release_query_snapshot(runtime, &before);
        result
    })
}
