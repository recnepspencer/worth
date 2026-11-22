//! Applied roots consume the real provider selection and complete Source custody.
use super::*;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

pub fn select_applied(
    runtime: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    interruption: super::super::super::ReadInterruption<'_>,
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    let refused = || path_union::traversal_denial(plan.name);
    let rows = plan.applied_managed_rows.ok_or_else(refused)?;
    let candidate = runtime.candidate_identity();
    if plan.managed_rows.is_some()
        || !rows.admits(plan.parent_session, plan.binding, plan.root, candidate)
    {
        return Err(refused());
    }
    let result = select_roots(
        runtime,
        &runtime.project_candidate(),
        execution,
        graph,
        plan,
        AdmittedRootSelection {
            roots: rows.roots,
            source: DeclaredManagedIndexSelection {
                access: Arc::clone(rows.association.access_contract()),
                dependencies: Arc::clone(rows.dependencies),
                coverage: rows
                    .coverage
                    .cloned()
                    .map(|coverage| coverage as Arc<dyn std::any::Any + Send + Sync>),
            },
        },
        true,
        None,
        interruption,
    )?;
    if !rows.admits(plan.parent_session, plan.binding, plan.root, candidate) {
        return Err(refused());
    }
    Ok(result)
}
