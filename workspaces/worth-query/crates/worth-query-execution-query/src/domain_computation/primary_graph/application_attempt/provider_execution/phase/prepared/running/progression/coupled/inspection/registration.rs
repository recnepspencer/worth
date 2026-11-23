//! One nominal mutation admission retains its real registered session through inspection.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::RegisteredProviderAttemptRuntimeExt as _;
pub struct InspectedRegistration<'run> {
    pub custody: CoupledWriterCustody,
    pub inspection: crate::domain_computation::WorthQueryProposedStateInspection<'run>,
    pub source: Option<crate::domain_computation::primary_graph::application_query::AppliedSourcePublication>,
    pub coupled_outputs: crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemands,
}
pub fn inspect_registered<'run, S, M>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    registered: RegisteredCoupledWriter<'run>,
    candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
        worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    >,
    own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
) -> Result<InspectedRegistration<'run>, WorthQueryApplicationCommitOutcome>
where
    S: ApplicationSchema,
    M: ApplicationMutationBinding<S>,
{
    let RegisteredCoupledWriter {
        source,
        coupled_outputs,
        typed,
        registered,
        returned,
        cleanup,
        lease,
        original_completion,
        completion_storage,
    } = registered;
    let authority = typed
        .downcast_ref::<CoupledWriterAdmission<S, M>>()
        .expect("installed executor retains its own registered admission");
    let request = authority.admission.publication_request().clone();
    let inspected = registered.inspect_coupled(
        execution,
        &application.runtime.primary_provider,
        &request,
        candidate,
        own_outputs,
        execution.allocation_policy(),
    );
    let (inspection, dispatch_outbox) = match inspected {
        Ok(inspected) => inspected,
        Err(outcome) => {
            drop(typed);
            return Err(progressed::finish_coupled_application_commit(
                execution,
                phase,
                application,
                completion_storage.complete(WorthQueryProgressedApplicationCommit {
                    original_completion,
                    outcome,
                    workflow_settlement_publication: None,
                    lease,
                    running: returned
                        .take()
                        .expect("closed inspection returned its actual run"),
                    cleanup,
                    request,
                }),
            ));
        }
    };
    Ok(InspectedRegistration {
        source,
        coupled_outputs,
        inspection,
        custody: CoupledWriterCustody {
            completion: None,
            typed,
            dispatch_outbox,
            returned,
            cleanup,
            lease,
            request,
            original_completion,
            completion_storage,
            own_outputs: None,
        },
    })
}
