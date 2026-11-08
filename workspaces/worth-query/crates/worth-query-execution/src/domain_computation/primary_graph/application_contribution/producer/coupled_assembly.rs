//! Declared roots select installed producers; Native decides their exact roster.
use super::execution::{CoupledSourceReadStop, PreparedCoupledWriter};
use super::*;
mod disposal;
mod preparation;
use crate::domain_computation::primary_graph::{
    application_attempt::coupled_outputs::DeclaredCoupledOutputDemands, WorthQueryAdvancementPhase,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use std::cell::RefCell;
use std::ops::ControlFlow;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_relational::facade::mvcc::{
    RelationalAuthoredCoupledCandidate, RelationalAuthoredCoupledProducerAssembly,
    RelationalCoupledAssemblyFailure,
};

pub(in crate::domain_computation::primary_graph) struct DeclaredProducerAssembly<
    'a,
    'run,
    'phase,
    S: ApplicationSchema,
> {
    phase: &'run WorthQueryAdvancementPhase<'phase>,
    coordination: &'a crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'a>,
    pub(super) application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    pub(super) principal: Option<&'a WorthQueryAuthenticatedExternalPrincipal<S>>,
    pub(super) before: &'a crate::basis::WorthQueryProductBranchLease,
    pub(super) request: &'a WorthQueryRequestScope,
    demands: &'a DeclaredCoupledOutputDemands,
    root_session: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    writers: RefCell<Vec<CoupledWriter<'run, S>>>,
    denial: RefCell<Option<WorthQueryOutputDemandDenial>>,
    abort_failures: RefCell<Vec<crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted>>,
}
struct CoupledWriter<'run, S> {
    producer: std::sync::Arc<registry::InstalledProducerProvider<S>>,
    stage: Option<CoupledWriterStage<'run>>,
    meaning: worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
}
enum CoupledWriterStage<'run> {
    Inspected(
        crate::domain_computation::primary_graph::application_attempt::InspectedCoupledWriter<'run>,
    ),
    Prepared(
        crate::domain_computation::primary_graph::application_attempt::CommitPreparedCoupledWriter<
            'run,
        >,
    ),
}
impl<'a, 'run, 'phase, S: ApplicationSchema> DeclaredProducerAssembly<'a, 'run, 'phase, S> {
    pub(in crate::domain_computation::primary_graph) fn new(
        phase: &'run WorthQueryAdvancementPhase<'phase>,
        coordination: &'a crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'a>,
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        principal: Option<&'a WorthQueryAuthenticatedExternalPrincipal<S>>,
        before: &'a crate::basis::WorthQueryProductBranchLease,
        request: &'a WorthQueryRequestScope,
        demands: &'a DeclaredCoupledOutputDemands,
        root_session: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    ) -> Self {
        Self {
            phase,
            coordination,
            application,
            principal,
            before,
            request,
            demands,
            root_session,
            writers: RefCell::new(Vec::new()),
            denial: RefCell::new(None),
            abort_failures: RefCell::new(Vec::new()),
        }
    }
    pub(super) fn retain_discovery_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: worth_relational::facade::identity::EntityId,
        support: crate::domain_computation::primary_graph::application_query::AppliedQuerySupport,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.application
            .runtime
            .primary_provider
            .retain_candidate_read_support(
                execution,
                &self.root_session,
                source,
                crate::domain_computation::primary_graph::provider::CandidateReadSupport::Query(
                    support,
                ),
            )
            .map_err(super::execution::denial::candidate_support_denied)
    }
    pub(super) fn retain_verified_parent(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        parent: super::VerifiedAppliedParent,
    ) -> Result<super::synchronous_outputs::AppliedOutputParent, WorthQueryOutputDemandDenial> {
        let applied = super::synchronous_outputs::AppliedOutputParent::new(
            parent.pin().clone(),
            parent.source(),
        );
        self.application
            .runtime
            .primary_provider
            .retain_candidate_read_support(
                execution,
                &self.root_session,
                parent.source(),
                crate::domain_computation::primary_graph::provider::CandidateReadSupport::Parent(
                    parent,
                ),
            )
            .map_err(super::execution::denial::candidate_support_denied)?;
        Ok(applied)
    }
    pub(in crate::domain_computation::primary_graph) fn take_denial(
        &self,
    ) -> Option<WorthQueryOutputDemandDenial> {
        self.denial.borrow_mut().take()
    }
    pub(in crate::domain_computation::primary_graph) fn settle_writers(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome{
        use crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome as Progression;
        use crate::domain_computation::primary_graph::{
            WorthQueryApplicationCommitOutcome as Outcome,
            WorthQueryApplicationUncommitted as Uncommitted,
        };
        let unpublished = match &root {
            Progression::ProductUnpublished(value) => Some(value),
            _ => None,
        };
        let mut failure = None;
        for writer in self.writers.borrow_mut().drain(..) {
            let Some(CoupledWriterStage::Prepared(prepared)) = writer.stage else {
                unreachable!("whole candidate prepares each writer before World")
            };
            let terminal = prepared.terminal_binding();
            let completion = prepared.completion();
            let provider = &self.application.runtime.primary_provider;
            let effected = unpublished.is_some()
                || provider
                    .observe_completed_application_for_session(&terminal)
                    .is_some()
                || provider
                    .inspect_pending_application_idempotency(
                        self.before.observation().lifecycle_incarnation(),
                    )
                    .is_some();
            let outcome = if effected {
                writer.producer.executor.complete_coupled_writer(
                    execution,
                    self.phase,
                    self.application,
                    prepared,
                    unpublished,
                )
            } else {
                self.retain_abort_outcome(prepared.abort(execution, self.phase, self.application));
                continue;
            };
            let uncommitted = match outcome {
                Outcome::Committed(receipt) | Outcome::AlreadyCommitted(receipt) => {
                    completion.complete(receipt);
                    continue;
                }
                Outcome::ProductUnpublished(_) => continue,
                other => match other.landed() {
                    Err(failure) => failure,
                    Ok(_) => unreachable!("only an uncommitted participant outcome remains"),
                },
            };
            failure = Some(match (failure.take(), uncommitted) {
                (Some(prior), Uncommitted::CleanupRequired(cleanup)) => {
                    let (_, cleanup) = cleanup.into_parts();
                    Uncommitted::CleanupRequired(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCleanupRequired::from_parts(prior, cleanup))
                }
                (Some(prior), _) => prior,
                (None, failure) => failure,
            });
        }
        self.finish_abort(failure.map(Progression::CoupledFailure).unwrap_or(root))
    }
    pub(in crate::domain_computation::primary_graph) fn authorize_members(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        for writer in self.writers.borrow().iter() {
            let Some(CoupledWriterStage::Prepared(approved)) = writer.stage.as_ref() else {
                unreachable!("validated group contains approved writers")
            };
            writer.producer.executor.authorize_coupled_writer(
                execution,
                self.application,
                approved,
                coordination,
            )?;
        }
        Ok(())
    }
}
impl<S: ApplicationSchema> RelationalAuthoredCoupledProducerAssembly
    for DeclaredProducerAssembly<'_, '_, '_, S>
{
    fn assemble(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        if !self.abort_writers(execution) {
            return Ok(ControlFlow::Break(()));
        }
        self.denial.borrow_mut().take();
        let prepared = self
            .application
            .runtime
            .primary_provider
            .reset_candidate_read_support(&self.root_session)
            .map_err(|cause| {
                CoupledSourceReadStop::Query(
                    super::execution::denial::candidate_support_denied(cause).into(),
                )
            })
            .and_then(|()| self.prepare(execution, candidate));
        match prepared {
            Ok(()) => Ok(ControlFlow::Continue(())),
            Err(CoupledSourceReadStop::Native(cause)) => Err(cause),
            Err(CoupledSourceReadStop::Query(cause)) => {
                *self.denial.borrow_mut() = Some(cause.into_output_denial());
                Ok(ControlFlow::Break(()))
            }
        }
    }
}

impl<S: ApplicationSchema>
    crate::domain_computation::primary_graph::provider::CoupledProducerAssembly
    for DeclaredProducerAssembly<'_, '_, '_, S>
{
    fn prepared_members(
        &self,
    ) -> Vec<crate::domain_computation::primary_graph::provider::CoupledApplicationMember> {
        self.writers
            .borrow()
            .iter()
            .map(|writer| {
                let Some(CoupledWriterStage::Prepared(prepared)) = writer.stage.as_ref() else {
                    unreachable!("validated group prepares every actual participant")
                };
                prepared.member(
                    &writer.producer.declaration.identity,
                    writer.meaning.clone(),
                )
            })
            .collect()
    }
    fn admit_validated(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        candidate: &worth_relational::facade::mvcc::ValidatedRelationalProposal,
    ) -> Result<(), crate::domain_computation::WorthQueryInvariantExecutionFailure> {
        for writer in self.writers.borrow_mut().iter_mut() {
            let Some(CoupledWriterStage::Inspected(inspected)) = writer.stage.as_ref() else {
                unreachable!("each actual assembly is approved once")
            };
            provider.admit_coupled_candidate(
                execution,
                inspected.session(),
                candidate,
                &writer.meaning,
            )?;
            let Some(CoupledWriterStage::Inspected(inspected)) = writer.stage.take() else {
                unreachable!("retained actual inspection")
            };
            let prepared = inspected
                .approve(execution, self.phase, self.application)
                .and_then(|approved| {
                    writer.producer.executor.prepare_coupled_commit(
                        execution,
                        self.phase,
                        self.application,
                        approved,
                    )
                });
            match prepared {
                Ok(approved) => writer.stage = Some(CoupledWriterStage::Prepared(approved)),
                Err(outcome) => {
                    let denial = super::execution::coupled_preparation_denial(
                        &writer.meaning.family,
                        outcome,
                    )
                    .into_output_denial();
                    let detail = denial.subject().to_owned();
                    *self.denial.borrow_mut() = Some(denial);
                    return Err(crate::domain_computation::WorthQueryInvariantExecutionFailure::new(
                        crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderRejected, detail,
                    ));
                }
            }
        }
        Ok(())
    }
}
