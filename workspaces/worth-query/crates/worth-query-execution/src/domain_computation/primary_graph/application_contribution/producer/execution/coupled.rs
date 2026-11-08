//! Installed producer dispatch retains the normal typed attempt through assembly.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    inspect_coupled_application, register_coupled_application, InspectedCoupledWriter,
};
use worth_relational::facade::{
    mvcc::{RelationalAuthoredCoupledCandidate, RequiredOutputPublication},
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
    transactions::WorkerIntentBatch,
};

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct PreparedCoupledWriter<
    'run,
> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) writer:
        InspectedCoupledWriter<'run>,
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) publication:
        RequiredOutputPublication,
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) batch:
        WorkerIntentBatch,
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) content:
        worth_relational::facade::content::RelationalContentAttachments,
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) trait InstalledCoupledProducerExecutor<
    Schema,
>
{
    fn prepare_coupled_commit<'run>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        writer: crate::domain_computation::primary_graph::application_attempt::ApprovedCoupledWriter<'run>,
    ) -> Result<
        crate::domain_computation::primary_graph::application_attempt::CommitPreparedCoupledWriter<
            'run,
        >,
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome,
    >;
    fn complete_coupled_writer(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        writer: crate::domain_computation::primary_graph::application_attempt::CommitPreparedCoupledWriter<'_>,
        unpublished: Option<&crate::domain_computation::WorthQueryProductUnpublishedApplication>,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;
    fn authorize_coupled_writer(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        writer: &crate::domain_computation::primary_graph::application_attempt::CommitPreparedCoupledWriter<'_>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>;

    fn read_coupled_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        before: &crate::basis::WorthQueryProductBranchLease,
        before_pin: &RelationalPinnedRead,
        demand: &crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<Option<ErasedAppliedProducerSource>, CoupledSourceReadStop>;

    fn prepare_coupled_writer<'run>(
        &self,
        phase: &'run WorthQueryAdvancementPhase<'_>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        before: &crate::basis::WorthQueryProductBranchLease,
        before_pin: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: ErasedAppliedProducerSource,
        edition: super::super::InstalledProducerEdition,
    ) -> Result<PreparedCoupledWriter<'run>, ProducerExecutionStop>;
}
impl<Schema, Binding> InstalledCoupledProducerExecutor<Schema>
    for TypedInstalledProducer<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, Binding>,
        >,
{
    fn prepare_coupled_commit<'run>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        writer: crate::domain_computation::primary_graph::application_attempt::ApprovedCoupledWriter<'run>,
    ) -> Result<
        crate::domain_computation::primary_graph::application_attempt::CommitPreparedCoupledWriter<
            'run,
        >,
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome,
    > {
        writer.prepare::<Schema, Binding>(execution, phase, runtime)
    }
    fn complete_coupled_writer(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        writer: crate::domain_computation::primary_graph::application_attempt::CommitPreparedCoupledWriter<'_>,
        unpublished: Option<&crate::domain_computation::WorthQueryProductUnpublishedApplication>,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome {
        writer.complete::<Schema, Binding>(execution, phase, runtime, unpublished)
    }
    fn authorize_coupled_writer(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        writer: &crate::domain_computation::primary_graph::application_attempt::CommitPreparedCoupledWriter<'_>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        writer.authorize::<Schema, Binding>(execution, runtime, coordination)
    }
    fn read_coupled_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        before: &crate::basis::WorthQueryProductBranchLease,
        before_pin: &RelationalPinnedRead,
        demand: &crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<Option<ErasedAppliedProducerSource>, CoupledSourceReadStop> {
        coupled_source::read::<Schema, Binding>(
            self, execution, runtime, principal, request, before, before_pin, demand, candidate,
        )
    }
    fn prepare_coupled_writer<'run>(
        &self,
        phase: &'run WorthQueryAdvancementPhase<'_>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        before: &crate::basis::WorthQueryProductBranchLease,
        before_pin: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: ErasedAppliedProducerSource,
        edition: super::super::InstalledProducerEdition,
    ) -> Result<PreparedCoupledWriter<'run>, ProducerExecutionStop> {
        let predecessor = source.predecessor.clone();
        let execution = phase
            .checked_execution_request_for(&runtime.runtime.product_runtime)
            .map_err(|stop| {
                denial(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(stop),
                    Binding::IDENTITY,
                )
            })?;
        let prepared = coupled_preparation::prepare::<Schema, Binding>(
            self,
            phase,
            coordination,
            runtime,
            principal,
            request,
            before,
            before_pin,
            candidate,
            source,
            predecessor.as_ref(),
            edition,
        )?;
        let registered =
            register_coupled_application::<Schema, Binding>(&execution, phase, runtime, prepared)
                .map_err(|outcome| {
                coupled_preparation::preparation_denial(Binding::IDENTITY, outcome)
            })?;
        let (writer, publication, batch, content) = inspect_coupled_application::<Schema, Binding>(
            &execution,
            phase,
            runtime,
            registered,
            candidate,
            predecessor,
        )
        .map_err(|outcome| coupled_preparation::preparation_denial(Binding::IDENTITY, outcome))?;
        Ok(PreparedCoupledWriter {
            writer,
            publication,
            batch,
            content,
        })
    }
}
