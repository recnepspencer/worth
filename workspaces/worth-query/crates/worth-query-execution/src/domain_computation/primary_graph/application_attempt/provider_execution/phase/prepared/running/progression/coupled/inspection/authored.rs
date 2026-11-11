//! An authored stage retains its mutation session, never producer publication custody.
use super::*;
pub(in crate::domain_computation::primary_graph) struct InspectedAuthoredApplication<'run> {
    pub(in crate::domain_computation::primary_graph) writer: InspectedCoupledWriter<'run>,
    pub(in crate::domain_computation::primary_graph) batch: worth_relational::facade::transactions::WorkerIntentBatch,
    pub(in crate::domain_computation::primary_graph) content: worth_relational::facade::content::RelationalContentAttachments,
    pub(in crate::domain_computation::primary_graph) roles: Vec<worth_relational::facade::mvcc::RequiredOutputRoleTarget>,
    pub(in crate::domain_computation::primary_graph) demands: crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemands,
}
pub(in crate::domain_computation::primary_graph) fn inspect_authored_application<'run, S, M>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    registered: RegisteredCoupledWriter<'run>,
    candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
        worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    >,
) -> Result<InspectedAuthoredApplication<'run>, WorthQueryApplicationCommitOutcome>
where
    S: ApplicationSchema,
    M: ApplicationMutationBinding<S>,
{
    let registration::InspectedRegistration {
        custody,
        inspection,
        source,
        coupled_outputs,
    } = registration::inspect_registered::<S, M>(
        execution,
        phase,
        application,
        registered,
        candidate,
    )?;
    let typed = custody
        .typed
        .downcast_ref::<CoupledWriterAdmission<S, M>>()
        .expect("stage executor retains its nominal mutation admission");
    let commands = if source.is_none() && typed.idempotency.authored_stage_identity().is_some() {
        application
            .runtime
            .primary_provider
            .authored_stage_commands(inspection.provider_session_view())
    } else {
        Err(crate::domain_computation::WorthQueryInvariantExecutionFailure::missing_coupled_settlement(typed.admission.scope_entity_id()))
    };
    match commands {
        Ok((batch, content, roles)) => Ok(InspectedAuthoredApplication {
            writer: InspectedCoupledWriter {
                custody,
                inspection,
            },
            batch,
            content,
            roles,
            demands: coupled_outputs,
        }),
        Err(cause) => {
            inspection.discard();
            let denial = crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::invariant_execution_denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution, cause,
            );
            Err(custody.finish(
                execution,
                phase,
                application,
                WorthQueryProviderProgressionOutcome::Denied(denial),
            ))
        }
    }
}
