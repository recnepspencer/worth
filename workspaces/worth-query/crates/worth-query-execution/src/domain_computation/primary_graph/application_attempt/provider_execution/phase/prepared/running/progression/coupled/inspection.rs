//! A registered writer materializes its own real effect overlay.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationProducerBinding;
mod authored;
mod authorization;
mod preparation;
mod registration;
pub(in crate::domain_computation::primary_graph) use authored::{
    inspect_authored_application, InspectedAuthoredApplication,
};
pub(in crate::domain_computation::primary_graph) use preparation::CommitPreparedCoupledWriter;

struct CoupledWriterCustody {
    completion:
        Option<crate::domain_computation::primary_graph::output_lineage::CoupledOutputCompletion>,
    typed: Box<dyn Any>,
    dispatch_outbox:
        Option<crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord>,
    returned: crate::domain_computation::provider_session::ReturnedProviderDirectRun,
    cleanup: WorthQueryApplicationMutationCleanupOwner,
    lease: WorthQueryApplicationSnapshotLease,
    request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
}
pub(in crate::domain_computation::primary_graph) struct InspectedCoupledWriter<'run> {
    custody: CoupledWriterCustody,
    inspection: crate::domain_computation::WorthQueryProposedStateInspection<'run>,
}
pub(in crate::domain_computation::primary_graph) struct ApprovedCoupledWriter<'run> {
    custody: CoupledWriterCustody,
    candidate: crate::domain_computation::WorthQueryInvariantApprovedProposedState<'run>,
}

pub(in crate::domain_computation::primary_graph) fn inspect_coupled_application<'run, S, B>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    registered: RegisteredCoupledWriter<'run>,
    candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
        worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    >,
    predecessor: Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
) -> Result<
    (
        InspectedCoupledWriter<'run>,
        worth_relational::facade::mvcc::RelationalCoupledProducerPosture,
        worth_relational::facade::transactions::WorkerIntentBatch,
        worth_relational::facade::content::RelationalContentAttachments,
    ),
    WorthQueryApplicationCommitOutcome,
>
where
    S: ApplicationSchema,
    B: WorthQueryApplicationProducerBinding<S>,
{
    use crate::domain_computation::primary_graph::WorthQueryProducerOutputFamily;
    let registration::InspectedRegistration {
        source,
        coupled_outputs: _,
        inspection,
        custody:
            CoupledWriterCustody {
                completion: _,
                mut typed,
                dispatch_outbox,
                returned,
                cleanup,
                lease,
                request,
            },
    } = registration::inspect_registered::<S, B::Operation>(
        execution,
        phase,
        application,
        registered,
        candidate,
    )?;
    let source = source.expect("installed producer retains its source publication");
    let handler = typed
        .downcast_mut::<CoupledWriterAdmission<S, B::Operation>>()
        .expect("installed writer keeps its nominal admission")
        .admission
        .graph_work_mut()
        .take_coupled_handler()
        .expect("coupled handler was armed and completed before registration");
    let commands = application
        .runtime
        .primary_provider
        .retain_coupled_preparation(
            execution,
            inspection.provider_session_view(),
            source,
            handler,
        )
        .and_then(|completion| {
            application
                .runtime
                .primary_provider
                .coupled_writer_commands(
                    execution,
                    lease.product().relational_basis(),
                    inspection.provider_session_view(),
                    B::IDENTITY,
                    B::OutputFamily::IDENTITY,
                    predecessor,
                )
                .map(|commands| (commands, completion))
        });
    let (commands, completion) = match commands {
        Ok(commands) => commands,
        Err(cause) => {
            inspection.discard();
            drop(typed);
            let outcome = WorthQueryProviderProgressionOutcome::Denied(crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::invariant_execution_denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution, cause));
            return Err(progressed::finish_application_commit(
                execution,
                phase,
                application,
                WorthQueryProgressedApplicationCommit {
                    outcome,
                    workflow_settlement_publication: None,
                    lease,
                    running: returned
                        .take()
                        .expect("discarded inspection returned its actual run"),
                    cleanup,
                    request,
                },
            ));
        }
    };
    Ok((
        InspectedCoupledWriter {
            custody: CoupledWriterCustody {
                completion: Some(completion),
                typed,
                dispatch_outbox,
                returned,
                cleanup,
                lease,
                request,
            },
            inspection,
        },
        commands.publication,
        commands.batch,
        commands.content,
    ))
}

impl<'run> InspectedCoupledWriter<'run> {
    pub(in crate::domain_computation::primary_graph) fn session(
        &self,
    ) -> crate::domain_computation::WorthQueryProviderSessionView<'_> {
        self.inspection.provider_session_view()
    }
    pub(in crate::domain_computation::primary_graph) fn approve<S: ApplicationSchema>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<ApprovedCoupledWriter<'run>, WorthQueryApplicationCommitOutcome> {
        let Self {
            custody,
            inspection,
        } = self;
        match super::super::invariant::complete_coupled_invariants(inspection, &custody.request) {
            Ok(candidate) => Ok(ApprovedCoupledWriter { custody, candidate }),
            Err(outcome) => Err(custody.finish(execution, phase, application, outcome)),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn abort<S: ApplicationSchema>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> WorthQueryApplicationCommitOutcome {
        self.inspection.discard();
        self.custody.finish(
            execution,
            phase,
            application,
            WorthQueryProviderProgressionOutcome::Aborted,
        )
    }
}

impl CoupledWriterCustody {
    fn finish<S: ApplicationSchema>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        outcome: WorthQueryProviderProgressionOutcome,
    ) -> WorthQueryApplicationCommitOutcome {
        let Self {
            completion: _,
            typed,
            dispatch_outbox: _,
            returned,
            cleanup,
            lease,
            request,
        } = self;
        drop(typed);
        progressed::finish_coupled_application_commit(
            execution,
            phase,
            application,
            WorthQueryProgressedApplicationCommit {
                outcome,
                workflow_settlement_publication: None,
                lease,
                running: returned
                    .take()
                    .expect("closed coupled protocol returns actual run"),
                cleanup,
                request,
            },
        )
    }
}
