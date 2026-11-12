//! Applied producer preparation reuses the selected conventional admission spine.
use super::*;
pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn prepare_applied_public_mutation<
    'selected,
    'runtime,
    S,
    B,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    installed: &TypedInstalledProducer<S, B>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    current: &'selected SharedSelectedProductOperation<'runtime, S>,
    before: &worth_relational::facade::runtime::RelationalPinnedRead,
    candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
        worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    >,
    external: &WorthQueryAuthenticatedExternalPrincipal<S>,
    request: &WorthQueryRequestScope,
    input: &MutationInput<S, B>,
) -> Result<PreparedSelectedPublicMutation<'selected, 'runtime, S, B>, SelectedPublicPreparationStop>
where
    S: ApplicationSchema + 'static,
    B: WorthQueryApplicationProducerBinding<S>,
    MutationScope<S, B>: ApplicationEntityMarkerIdentity<S>,
{
    prepare_public_mutation(
        execution,
        installed,
        runtime,
        current,
        None,
        None,
        external,
        request,
        input,
        None,
        Some((before, candidate)),
    )
}
