//! One actual direct-run authority check and provider contract construction.
use super::*;
pub(super) fn prepare(
    run: &WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> Result<
    (
        WorthQueryProviderExecutionPlanContract,
        Arc<WorthQueryGraphProviderAnchor>,
        WorthQueryProviderSessionProtocolCounters,
    ),
    WorthQueryProviderSessionFailure,
> {
    let mut counters = WorthQueryProviderSessionProtocolCounters::default();
    let operation = run.provider_plan_operation();
    let session = run.provider_plan_session();
    let (resources, evidence) = run.provider_plan_resources();
    counters.checked_authority();
    validate_common_authority(
        WorthQueryProviderPlanAuthorityObservation {
            operation,
            session_identity: session.identity(),
            session_attempt_identity: session.attempt_identity(),
            resource_authority_matches: operation.admits_provider_plan_resources(None, resources),
            evidence_session_identity: evidence.provider_session_identity(),
            evidence_attempt_identity: evidence.provider_session_attempt_identity(),
            bridge: run.provider_plan_bridge_basis(),
            graph,
            stage_identity: None,
        },
        &counters,
    )?;
    let provider = retain_session_provider(graph, &counters)?;
    let snapshot_identity = run.execution_snapshot_reference();
    let contract = operation
        .provider_plan_contract(WorthQueryValidatedProviderPlan {
            operation,
            stage_identity: None,
            managed_run_identity: run.identity(),
            execution_basis_identity: run.provider_plan_bridge_basis().identity().as_str(),
            admitted_session_identity: session.identity(),
            resource_attempt_identity: session.attempt_identity(),
            graph,
            snapshot_identity: &snapshot_identity,
            resource_envelope_identity: resources.envelope_identity(),
            provider_identity: provider.provider_identity(),
            provider_generation: provider.provider_generation(),
        })
        .ok_or_else(|| undeclared_scope(&counters))?;
    counters.bound_closure_items(contract.closure_width());
    Ok((contract, provider, counters))
}
