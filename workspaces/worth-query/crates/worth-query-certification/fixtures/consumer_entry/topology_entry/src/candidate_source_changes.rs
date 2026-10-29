//! Compiled public DX for retaining actual pre-evaluation source custody.
use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantDerivedRelationBinding,
    WorthQueryApplicationInvariantScopePlanner, WorthQueryInvariantAccessDenial,
    WorthQueryPreparedInvariantDerivedChanges,
};

// compiled-example: begin
pub fn prepare_changes<S, R, F, T>(
    planner: &WorthQueryApplicationInvariantScopePlanner<'_, '_, S>,
    binding: &WorthQueryApplicationInvariantDerivedRelationBinding<S, R, F, T>,
) -> Result<WorthQueryPreparedInvariantDerivedChanges<S, R, F, T>, WorthQueryInvariantAccessDenial>
{
    planner.prepare_derived_relation_changes(binding)
}

pub fn read_changes<S, R, F, T>(
    context: &WorthQueryApplicationInvariantContext<'_, '_, S>,
    binding: &WorthQueryApplicationInvariantDerivedRelationBinding<S, R, F, T>,
    prepared: &WorthQueryPreparedInvariantDerivedChanges<S, R, F, T>,
) -> Result<usize, WorthQueryInvariantAccessDenial> {
    Ok(context
        .derived_relation_changes(binding, prepared)?
        .affected_roots()
        .len())
}
// compiled-example: end
