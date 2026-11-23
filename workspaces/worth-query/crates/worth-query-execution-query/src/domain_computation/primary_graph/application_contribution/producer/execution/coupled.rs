//! Installed producer dispatch retains the normal typed attempt through assembly.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    inspect_coupled_application, register_coupled_application, InspectedCoupledWriter,
};
use worth_relational::facade::{
    mvcc::{RelationalAuthoredCoupledCandidate, RelationalCoupledProducerPosture},
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
    transactions::WorkerIntentBatch,
};

pub struct PreparedCoupledWriter<'run> {
    pub writer: InspectedCoupledWriter<'run>,
    pub publication: RelationalCoupledProducerPosture,
    pub batch: WorkerIntentBatch,
    pub content: worth_relational::facade::content::RelationalContentAttachments,
}

pub trait InstalledCoupledProducerExecutor<Schema> {
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
        purpose: CoupledSourceReadPurpose,
    ) -> Result<CoupledSourceRead, CoupledSourceReadStop>;

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
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
        participants: &dyn crate::domain_computation::primary_graph::provider::AppliedOutputParticipants,
        assembly: &dyn crate::domain_computation::primary_graph::application_query::managed_index::AppliedManagedSourceAlignment,
        source: ErasedAppliedProducerSource,
        edition: super::super::InstalledProducerEdition,
        captured: Option<(Box<dyn CapturedProducerInputRead>, std::any::TypeId)>,
    ) -> Result<PreparedCoupledWriter<'run>, ProducerExecutionStop>;

    fn capture_coupled_inputs(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        before: &crate::basis::WorthQueryProductBranchLease,
        before_pin: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: ErasedAppliedProducerSource,
        edition: super::super::InstalledProducerEdition,
        component: std::any::TypeId,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
    ) -> Result<Box<dyn CapturedProducerInputRead>, ProducerExecutionStop>;
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
        writer.prepare::<Schema, Binding::Operation>(execution, phase, runtime)
    }
    fn complete_coupled_writer(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        writer: crate::domain_computation::primary_graph::application_attempt::CommitPreparedCoupledWriter<'_>,
        unpublished: Option<&crate::domain_computation::WorthQueryProductUnpublishedApplication>,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome {
        writer.complete::<Schema, Binding::Operation>(execution, phase, runtime, unpublished)
    }
    fn authorize_coupled_writer(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        writer: &crate::domain_computation::primary_graph::application_attempt::CommitPreparedCoupledWriter<'_>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        writer.authorize::<Schema, Binding::Operation>(execution, runtime, coordination)
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
        purpose: CoupledSourceReadPurpose,
    ) -> Result<CoupledSourceRead, CoupledSourceReadStop> {
        coupled_source::read::<Schema, Binding>(
            self, execution, runtime, principal, request, before, before_pin, demand, candidate,
            purpose,
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
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
        participants: &dyn crate::domain_computation::primary_graph::provider::AppliedOutputParticipants,
        assembly: &dyn crate::domain_computation::primary_graph::application_query::managed_index::AppliedManagedSourceAlignment,
        source: ErasedAppliedProducerSource,
        edition: super::super::InstalledProducerEdition,
        captured: Option<(Box<dyn CapturedProducerInputRead>, std::any::TypeId)>,
    ) -> Result<PreparedCoupledWriter<'run>, ProducerExecutionStop> {
        let captured = captured
            .map(|(inputs, component)| {
                let inputs: Box<dyn std::any::Any> = inputs;
                inputs
                    .downcast::<super::input_preparation::PreparedProducerInputs<Schema, Binding>>()
                    .map(|inputs| (*inputs, component))
                    .map_err(|_| {
                        denial(
                            WorthQueryOutputDemandDenialKind::ForeignSource,
                            Binding::IDENTITY,
                        )
                    })
            })
            .transpose()?;
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
            own_outputs,
            participants,
            assembly,
            source,
            predecessor.as_ref(),
            edition,
            captured,
        )?;
        let registered = register_coupled_application::<Schema, Binding::Operation>(
            &execution, phase, runtime, prepared,
        )
        .map_err(|outcome| coupled_preparation::preparation_denial(Binding::IDENTITY, outcome))?;
        let (writer, publication, batch, content) = inspect_coupled_application::<Schema, Binding>(
            &execution,
            phase,
            runtime,
            registered,
            candidate,
            own_outputs,
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

    fn capture_coupled_inputs(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        before: &crate::basis::WorthQueryProductBranchLease,
        before_pin: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: ErasedAppliedProducerSource,
        edition: super::super::InstalledProducerEdition,
        component: std::any::TypeId,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
    ) -> Result<Box<dyn CapturedProducerInputRead>, ProducerExecutionStop> {
        let owner = self.input_capture.ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Binding::IDENTITY,
            )
        })?;
        (owner.admit)(
            self,
            phase,
            runtime,
            principal,
            request,
            before,
            before_pin,
            candidate,
            source,
            edition,
            component,
            own_outputs,
        )
    }
}
