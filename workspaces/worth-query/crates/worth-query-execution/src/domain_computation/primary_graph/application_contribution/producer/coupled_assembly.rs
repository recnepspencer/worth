//! Declared roots select installed producers; Native decides their exact roster.
use super::execution::{CoupledSourceReadStop, PreparedCoupledWriter};
use super::*;
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
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    principal: Option<&'a WorthQueryAuthenticatedExternalPrincipal<S>>,
    before: &'a crate::basis::WorthQueryProductBranchLease,
    request: &'a WorthQueryRequestScope,
    demands: &'a DeclaredCoupledOutputDemands,
    writers: RefCell<Vec<CoupledWriter<'run, S>>>,
    denial: RefCell<Option<WorthQueryOutputDemandDenial>>,
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
    ) -> Self {
        Self {
            phase,
            coordination,
            application,
            principal,
            before,
            request,
            demands,
            writers: RefCell::new(Vec::new()),
            denial: RefCell::new(None),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn take_denial(
        &self,
    ) -> Option<WorthQueryOutputDemandDenial> {
        self.denial.borrow_mut().take()
    }
    pub(in crate::domain_computation::primary_graph) fn abort_writers(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        for writer in self.writers.borrow_mut().drain(..) {
            if let Some(stage) = writer.stage {
                let _ = match stage {
                    CoupledWriterStage::Inspected(writer) => {
                        writer.abort(execution, self.phase, self.application)
                    }
                    CoupledWriterStage::Prepared(writer) => {
                        writer.abort(execution, self.phase, self.application)
                    }
                };
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn settle_writers(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome{
        use crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome as Progression;
        use crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted as Uncommitted;
        let unpublished = match &root {
            Progression::ProductUnpublished(value) => Some(value),
            Progression::CoupledFailure(_)
            | Progression::Committed(_)
            | Progression::RecoveredCommitted(_)
            | Progression::ProductPreparationPending(_)
            | Progression::ProductPreparationRefused(_)
            | Progression::ProductStale(_)
            | Progression::NoEffect(_)
            | Progression::Denied(_)
            | Progression::Indeterminate(_)
            | Progression::Stale(_)
            | Progression::Cancelled
            | Progression::TimedOut
            | Progression::Aborted
            | Progression::Deferred(_) => None,
        };
        let mut failure: Option<Uncommitted> = None;
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
                writer.producer.executor.complete_coupled_writer(
                    execution,
                    self.phase,
                    self.application,
                    prepared,
                    unpublished,
                )
            } else {
                prepared.abort(execution, self.phase, self.application)
            };
            let uncommitted = match outcome.landed() {
                Ok(landed) => {
                    completion.complete(landed);
                    continue;
                }
                Err(Uncommitted::ProductUnpublished(_)) => continue,
                Err(Uncommitted::Aborted) if !effected => continue,
                Err(
                    uncommitted @ (Uncommitted::CleanupRequired(_)
                    | Uncommitted::ProductStale(_)
                    | Uncommitted::ProductPreparationPending(_)
                    | Uncommitted::ProductPreparationRefused(_)
                    | Uncommitted::NoEffect(_)
                    | Uncommitted::Stale(_)
                    | Uncommitted::Cancelled
                    | Uncommitted::TimedOut
                    | Uncommitted::Denied(_)
                    | Uncommitted::Aborted
                    | Uncommitted::Deferred(_)
                    | Uncommitted::Indeterminate(_)),
                ) => uncommitted,
            };
            failure = Some(match (failure.take(), uncommitted) {
                (Some(prior), Uncommitted::CleanupRequired(cleanup)) => {
                    let (_, cleanup) = cleanup.into_parts();
                    Uncommitted::CleanupRequired(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCleanupRequired::from_parts(prior.into(), cleanup))
                }
                (
                    Some(prior),
                    Uncommitted::ProductStale(_)
                    | Uncommitted::ProductUnpublished(_)
                    | Uncommitted::ProductPreparationPending(_)
                    | Uncommitted::ProductPreparationRefused(_)
                    | Uncommitted::NoEffect(_)
                    | Uncommitted::Stale(_)
                    | Uncommitted::Cancelled
                    | Uncommitted::TimedOut
                    | Uncommitted::Denied(_)
                    | Uncommitted::Aborted
                    | Uncommitted::Deferred(_)
                    | Uncommitted::Indeterminate(_),
                ) => prior,
                (None, failure) => failure,
            });
        }
        failure.map(Progression::CoupledFailure).unwrap_or(root)
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
    fn prepare(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), CoupledSourceReadStop> {
        if let Some(cause) = self.demands.take_failure() {
            return Err(CoupledSourceReadStop::capture(cause));
        }
        let principal = self.principal.ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "coupled producer authentication",
            )
        })?;
        let graph = &self.application.runtime.primary_provider.graph;
        let before_pin = graph.with_runtime(execution, |runtime| {
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                runtime, self.before.relational_basis(),
            ).map_err(|cause| WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ForeignSource, format!("coupled before basis: {cause:?}")))?;
            let indexes = graph.layout.equality_index_ids().collect::<Vec<_>>();
            let pin = runtime.read_truth().pin_read_capability(&snapshot, &indexes, execution);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            pin.map_err(|cause| WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ForeignSource, format!("coupled before pin: {cause:?}")))
        })?;
        let registry = &self.application.installed_producers;
        for demand in &self.demands.values {
            let source_entry = registry.declared_coupled_source(demand)?;
            let Some(source) = source_entry.executor.read_coupled_source(
                execution,
                self.application,
                principal,
                self.request,
                self.before,
                &before_pin,
                demand,
                candidate,
            )?
            else {
                continue;
            };
            let producer = registry.declared_coupled_preserve(demand, source.profile)?;
            let pin = candidate.pin_source(execution)?;
            let prepared = producer.executor.prepare_coupled_writer(
                self.phase,
                self.coordination,
                self.application,
                principal,
                self.request,
                self.before,
                &before_pin,
                &pin,
                source,
                producer.edition,
            )?;
            let PreparedCoupledWriter {
                writer,
                publication,
                batch,
                content,
            } = prepared;
            // Keep the session even when Native requests an ordinary symbol retry;
            // the next actual assembly round closes it before rerunning the handler.
            let worth_relational::facade::mvcc::RequiredOutputPublication::Preserve {
                meaning, ..
            } = &publication
            else {
                unreachable!("installed coupled preparation issues Preserve")
            };
            self.writers.borrow_mut().push(CoupledWriter {
                producer: std::sync::Arc::clone(producer),
                stage: Some(CoupledWriterStage::Inspected(writer)),
                meaning: meaning.clone(),
            });
            match self
                .application
                .runtime
                .primary_provider
                .application_attempt_operation_control
                .take_coupled_participant_corruption()
            {
                fault @ (1..=3) => candidate.append_corrupt_participant_for_test(
                    execution,
                    publication,
                    &content,
                    vec![batch],
                    fault,
                )?,
                _ => candidate.append_preserve(execution, publication, &content, vec![batch])?,
            }
            let after_own = candidate.pin_source(execution)?;
            let writers = self.writers.borrow();
            let Some(CoupledWriterStage::Inspected(writer)) =
                writers.last().and_then(|writer| writer.stage.as_ref())
            else {
                unreachable!("actual appended writer")
            };
            self.application
                .runtime
                .primary_provider
                .retain_coupled_after_own(writer.session(), after_own);
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
        self.abort_writers(execution);
        self.denial.borrow_mut().take();
        match self.prepare(execution, candidate) {
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
