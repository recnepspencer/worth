//! One Native candidate owns the declared authored stages and required producer wave.
use super::producer::execution::{CoupledSourceReadStop, PreparedCoupledWriter};
use super::producer::{
    execution, registry, synchronous_outputs, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind,
};
use worth_query_installation::facade::ApplicationSchema;
mod authored_stages;
mod disposal;
mod preparation;
mod validation;
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

pub(in crate::domain_computation::primary_graph) struct CoupledApplicationAssembly<
    'a,
    'run,
    'phase,
    S: ApplicationSchema,
> {
    phase: &'run WorthQueryAdvancementPhase<'phase>,
    coordination: &'a crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'a>,
    pub(in crate::domain_computation::primary_graph::application_contribution) application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    pub(in crate::domain_computation::primary_graph::application_contribution) principal: Option<&'a WorthQueryAuthenticatedExternalPrincipal<S>>,
    pub(in crate::domain_computation::primary_graph::application_contribution) before: &'a crate::basis::WorthQueryProductBranchLease,
    pub(in crate::domain_computation::primary_graph::application_contribution) request: &'a WorthQueryRequestScope,
    demands: &'a DeclaredCoupledOutputDemands,
    root_session: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    rounds: RefCell<synchronous_outputs::rounds::limits::ProducerRoundBudget>,
    writers: RefCell<Vec<CoupledWriter<'run, S>>>,
    authored: RefCell<Vec<AuthoredWriter<'run, S>>>,
    denial: RefCell<Option<WorthQueryOutputDemandDenial>>,
    abort_failures: RefCell<Vec<crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted>>,
    /// The last assembly round ended the edit (`Break`).
    broke: std::cell::Cell<bool>,
}
struct AuthoredWriter<'run, S> {
    capture: crate::domain_computation::primary_graph::application_contribution::authored_edit::CapturedAuthoredEditStage<S>,
    physical: usize,
    position: usize,
    stage: Option<CoupledWriterStage<'run>>,
}
struct CoupledWriter<'run, S> {
    position: usize,
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
impl<'a, 'run, 'phase, S: ApplicationSchema> CoupledApplicationAssembly<'a, 'run, 'phase, S> {
    fn applied_position(&self) -> usize {
        self.authored.borrow().len() + self.writers.borrow().len()
    }
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
            rounds: RefCell::new(
                synchronous_outputs::rounds::limits::ProducerRoundBudget::new(demands.round_limits),
            ),
            writers: RefCell::new(Vec::new()),
            authored: RefCell::new(Vec::new()),
            denial: RefCell::new(None),
            abort_failures: RefCell::new(Vec::new()),
            broke: std::cell::Cell::new(false),
        }
    }
    pub(in crate::domain_computation::primary_graph::application_contribution) fn retain_discovery_support(
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
                self.applied_position(),
                crate::domain_computation::primary_graph::provider::CandidateReadSupport::Query(
                    support,
                ),
            )
            .map_err(execution::denial::candidate_support_denied)
    }
    pub(in crate::domain_computation::primary_graph::application_contribution) fn retain_verified_parent(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        parent: super::producer::VerifiedAppliedParent,
    ) -> Result<synchronous_outputs::AppliedOutputParent, WorthQueryOutputDemandDenial> {
        let applied =
            synchronous_outputs::AppliedOutputParent::new(parent.pin().clone(), parent.source());
        self.application
            .runtime
            .primary_provider
            .retain_candidate_read_support(
                execution,
                &self.root_session,
                parent.source(),
                self.applied_position(),
                crate::domain_computation::primary_graph::provider::CandidateReadSupport::Parent(
                    parent,
                ),
            )
            .map_err(execution::denial::candidate_support_denied)?;
        Ok(applied)
    }
    pub(in crate::domain_computation::primary_graph) fn take_denial(
        &self,
    ) -> Option<WorthQueryOutputDemandDenial> {
        self.denial.borrow_mut().take()
    }
    /// The last round ended the edit and left neither a refusing subject nor
    /// an abort failure for the caller: the refusal was lost.
    pub(in crate::domain_computation::primary_graph) fn broke_without_subject(&self) -> bool {
        self.broke.get()
            && self.denial.borrow().is_none()
            && self.abort_failures.borrow().is_empty()
    }
    pub(in crate::domain_computation::primary_graph) fn settle_writers(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome{
        use crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome as Progression;
        use crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted as Uncommitted;
        let unpublished = match &root {
            Progression::ProductUnpublished(value) => Some(value),
            _ => None,
        };
        let mut failure = self.settle_authored(execution, unpublished);
        for writer in self.writers.borrow_mut().drain(..) {
            let Some(CoupledWriterStage::Prepared(prepared)) = writer.stage else {
                unreachable!("whole candidate prepares each writer before World")
            };
            let completion = prepared.completion();
            let provider = &self.application.runtime.primary_provider;
            let effected = unpublished.is_some()
                || prepared.has_original_completion()
                || provider
                    .inspect_pending_application_idempotency(
                        self.before.observation().lifecycle_incarnation(),
                    )
                    .is_some();
            let outcome = if effected {
                writer.producer.coupled_executor().complete_coupled_writer(
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
            let uncommitted = match outcome.landed() {
                Ok(landed) => {
                    if let crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Live { receipt } = &landed {
                        execution::contact_observation::record_live_completion(
                            self.application, execution, &writer.producer.declaration.identity,
                            completion.source().root(), receipt, self.before,
                            match &root { Progression::Committed(receipt) => Some(receipt.original_publication()), _ => None },
                        );
                    }
                    completion.complete(landed);
                    continue;
                }
                Err(Uncommitted::ProductUnpublished(_)) => continue,
                Err(failure) => failure,
            };
            failure = Some(match (failure.take(), uncommitted) {
                (Some(prior), Uncommitted::CleanupRequired(cleanup)) => {
                    let (_, cleanup) = cleanup.into_parts();
                    Uncommitted::CleanupRequired(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCleanupRequired::from_parts(prior.into(), cleanup))
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
        for writer in self.authored.borrow().iter() {
            let Some(CoupledWriterStage::Prepared(prepared)) = writer.stage.as_ref() else {
                unreachable!("validated authored stage")
            };
            writer.capture.executor.authorize(
                prepared,
                execution,
                self.application,
                coordination,
            )?;
        }
        for writer in self.writers.borrow().iter() {
            let Some(CoupledWriterStage::Prepared(approved)) = writer.stage.as_ref() else {
                unreachable!("validated group contains approved writers")
            };
            writer
                .producer
                .coupled_executor()
                .authorize_coupled_writer(execution, self.application, approved, coordination)?;
        }
        Ok(())
    }
}
impl<S: ApplicationSchema> RelationalAuthoredCoupledProducerAssembly
    for CoupledApplicationAssembly<'_, '_, '_, S>
{
    fn assemble(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        // Native may abandon a candidate to load one of its 256 symbol buckets
        // and call this logical envelope again (at most 257 attempts). A real
        // refusal belongs to the envelope, so no later callback may erase it.
        if self.denial.borrow().is_some() {
            self.broke.set(true);
            return Ok(ControlFlow::Break(()));
        }
        self.broke.set(false);
        if !self.abort_writers(execution) {
            self.broke.set(true);
            return Ok(ControlFlow::Break(()));
        }
        if self
            .application
            .runtime
            .primary_provider
            .application_attempt_operation_control
            .take_break_without_subject()
        {
            self.broke.set(true);
            return Ok(ControlFlow::Break(()));
        }
        // A cold-symbol retry abandoned the complete candidate without an
        // effect (G4/D1619). Route rounds are per candidate, while the original
        // execution request still pays all work and Native bounds retries by
        // its 256 buckets. Cold placement cannot consume a logical route round.
        *self.rounds.borrow_mut() = synchronous_outputs::rounds::limits::ProducerRoundBudget::new(
            self.demands.round_limits,
        );
        let prepared = self
            .application
            .runtime
            .primary_provider
            .reset_candidate_read_support(&self.root_session)
            .map_err(|cause| {
                CoupledSourceReadStop::Query(
                    execution::denial::candidate_support_denied(cause).into(),
                )
            })
            .and_then(|()| self.prepare(execution, candidate));
        match prepared {
            Ok(()) => Ok(ControlFlow::Continue(())),
            Err(CoupledSourceReadStop::Native(cause)) => Err(cause),
            Err(CoupledSourceReadStop::Query(cause)) => {
                self.denial
                    .borrow_mut()
                    .get_or_insert_with(|| cause.into_output_denial());
                self.broke.set(true);
                Ok(ControlFlow::Break(()))
            }
        }
    }
}
