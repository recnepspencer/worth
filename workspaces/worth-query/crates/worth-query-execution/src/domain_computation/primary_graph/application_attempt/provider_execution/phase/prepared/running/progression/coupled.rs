//! Keep each actual producer's typed admission through the shared publication.
use super::super::super::{
    entry_custody::prepare_application_commit, WorthQueryApplicationCommitPreparation,
    WorthQueryApplicationCommitPreparationRequest,
};
use super::super::start_managed_application_commit;
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
};
use std::any::Any;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

type Operation<S, B> = <B as ApplicationMutationBinding<S>>::Operation;
type Input<S, B> = <B as ApplicationMutationBinding<S>>::Input;
type Scope<S, B> = <<B as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;

/// Erasure retains this nominal typed owner, including the complete admission
/// brand. Only the installed executor for the same binding can recover it.
pub(in crate::domain_computation::primary_graph) struct CoupledMutationAttempt<
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
> {
    commit: WorthQueryRunningApplicationCommit<S, Operation<S, B>, Input<S, B>, Scope<S, B>>,
    source: Option<
        crate::domain_computation::primary_graph::application_query::AppliedSourcePublication,
    >,
    marker: std::marker::PhantomData<fn() -> B>,
}

pub(in crate::domain_computation::primary_graph) fn prepare_coupled_application<S, B>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    program: WorthQueryApplicationEffectProgram<S, Operation<S, B>, Input<S, B>, Scope<S, B>>,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    source: Option<
        crate::domain_computation::primary_graph::application_query::AppliedSourcePublication,
    >,
) -> Result<Box<dyn Any>, WorthQueryApplicationCommitOutcome>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: Clone + Send + Sync,
{
    let entry = application.validate_commit_program_in_advancement(phase, program)?;
    let prepared = match prepare_application_commit(
        execution,
        WorthQueryApplicationCommitPreparationRequest::new(entry, idempotency, None, None)
            .under_coordination(coordination),
    ) {
        WorthQueryApplicationCommitPreparation::Ready(prepared) => prepared,
        WorthQueryApplicationCommitPreparation::Terminal(outcome) => return Err(outcome),
    };
    let commit = start_managed_application_commit(phase, application, prepared)?;
    Ok(Box::new(CoupledMutationAttempt::<S, B> {
        commit,
        source,
        marker: std::marker::PhantomData,
    }))
}

/// Session custody can borrow the outer advancement while the actual admission
/// remains nominally typed and owned by its installed producer executor.
pub(in crate::domain_computation::primary_graph) struct RegisteredCoupledWriter<'run> {
    source: Option<crate::domain_computation::primary_graph::application_query::AppliedSourcePublication>,
    coupled_outputs: crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemands,
    typed: Box<dyn Any>,
    registered: WorthQueryRegisteredProviderAttempt<'run>,
    returned: crate::domain_computation::provider_session::ReturnedProviderDirectRun,
    cleanup: WorthQueryApplicationMutationCleanupOwner,
    lease: WorthQueryApplicationSnapshotLease,
    original_completion:
        crate::domain_computation::primary_graph::provider::OriginalCompletionReceiver,
    completion_storage: super::super::completion_storage::ApplicationCommitCompletionStorage,
}
struct CoupledWriterAdmission<S: ApplicationSchema, B: ApplicationMutationBinding<S>> {
    admission: crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        S,
        Operation<S, B>,
        Input<S, B>,
        Scope<S, B>,
    >,
    authorization:
        crate::domain_computation::authorization::WorthQueryRegisteredCommitAuthorization,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    marker: std::marker::PhantomData<fn() -> B>,
}

pub(in crate::domain_computation::primary_graph) fn register_coupled_application<'run, S, B>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &'run WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    attempt: Box<dyn Any>,
) -> Result<RegisteredCoupledWriter<'run>, WorthQueryApplicationCommitOutcome>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: Clone + Send + Sync,
{
    let attempt = attempt
        .downcast::<CoupledMutationAttempt<S, B>>()
        .unwrap_or_else(|_| panic!("installed executor retains its own nominal producer attempt"));
    let source = attempt.source;
    let WorthQueryRunningApplicationCommit {
        coupled_outputs,
        completion_storage,
        admission,
        lease,
        provider_attempt,
        outcome_identity,
        mut authorization,
        idempotency,
        running,
        mutation_run,
        mut attempt_basis,
        workflow_settlement_publication,
        aftermath_causality,
        ..
    } = attempt.commit;
    let original_completion = attempt_basis.take_original_completion();
    let request = admission.publication_request().clone();
    let fail = |completion: WorthQueryProviderProgressionCompletion,
                running: crate::domain_computation::WorthQueryRunningDirectRun,
                lease,
                original_completion,
                completion_storage: super::super::completion_storage::ApplicationCommitCompletionStorage| {
        progressed::finish_coupled_application_commit(
            execution,
            phase,
            application,
            completion_storage.complete(completion.finish(
                lease, running, workflow_settlement_publication, request, original_completion,
            )),
        )
    };
    use session_admission::CoupledProviderSessionAdmission as Admission;
    let (session, returned) = match session_admission::admit_coupled_provider_session(
        execution,
        phase,
        running,
        application.runtime.primary_graph_authority(),
        attempt_basis.retained_product().retained_clone(),
        mutation_run,
    ) {
        Admission::Entered { session, returned } => (session, returned),
        Admission::AllocationDenied {
            run,
            cause,
            mutation_run,
        } => {
            let denial = crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::provider_rejected_with_detail(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::ResourceAdmission,
                format!("coupled provider run retention: {cause:?}"));
            return Err(fail(
                WorthQueryProviderProgressionCompletion {
                    outcome: WorthQueryProviderProgressionOutcome::Denied(denial),
                    cleanup: WorthQueryApplicationMutationCleanupOwner::Unbound(mutation_run),
                },
                run,
                lease,
                original_completion,
                completion_storage,
            ));
        }
    };
    let session = match session {
        Ok(session) => session,
        Err(failure) => {
            return Err(fail(
                failure.into_completion(),
                returned.take().expect("closed plan returned actual run"),
                lease,
                original_completion,
                completion_storage,
            ))
        }
    };
    let registered = match session.register(
        &mut authorization,
        provider_attempt,
        attempt_basis,
        WorthQueryProviderAttemptRegistrationContext::new(
            &application.runtime.primary_provider,
            &admission,
            idempotency,
            outcome_identity,
            aftermath_causality.as_ref(),
        ),
        execution.allocation_policy(),
    ) {
        Ok(registered) => registered,
        Err(failure) => {
            return Err(fail(
                failure.into_completion(),
                returned
                    .take()
                    .expect("closed registration returned actual run"),
                lease,
                original_completion,
                completion_storage,
            ))
        }
    };
    let Some(authorization) = authorization.finish_registration() else {
        let completion = registered.deny(crate::domain_computation::primary_graph::application_attempt::provider_execution::progression_denied(
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::DecisionReadSet));
        return Err(fail(
            completion,
            returned.take().expect("closed session returned actual run"),
            lease,
            original_completion,
            completion_storage,
        ));
    };
    let (registered, mutation_run) = registered.into_coupled_parts();
    Ok(RegisteredCoupledWriter {
        source,
        coupled_outputs,
        typed: Box::new(CoupledWriterAdmission::<S, B> {
            admission,
            authorization,
            idempotency,
            marker: std::marker::PhantomData,
        }),
        registered,
        returned,
        cleanup: WorthQueryApplicationMutationCleanupOwner::ProviderBound(mutation_run),
        lease,
        original_completion,
        completion_storage,
    })
}

mod inspection;
pub(in crate::domain_computation::primary_graph) use inspection::{
    inspect_authored_application, inspect_coupled_application, ApprovedCoupledWriter,
    CommitPreparedCoupledWriter, InspectedAuthoredApplication, InspectedCoupledWriter,
};
