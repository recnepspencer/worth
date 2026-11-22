//! Nominal typed stage factories remain typed behind request-local erasure.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    CommitPreparedCoupledWriter, InspectedAuthoredApplication, InspectedCoupledWriter,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::application_operation::{
    ApplicationEditEnvelopeKey, ApplicationEncodedInput,
};
use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

pub struct AuthoredEditStageAdmission<S: ApplicationSchema, B: ApplicationMutationBinding<S>> {
    pub(super) operation: AuthoredEditStageOperation<S, B>,
    pub(super) principal: B::PrincipalIdentity,
    pub(super) pending_source: Option<crate::domain_computation::primary_graph::WorthQueryPendingSourceExpectation<<B::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<S>>::Query>>,
}
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> AuthoredEditStageAdmission<S, B> {
    pub fn new(
        operation: AuthoredEditStageOperation<S, B>,
        principal: B::PrincipalIdentity,
    ) -> Self {
        Self {
            operation,
            principal,
            pending_source: None,
        }
    }
    pub fn with_source_expectation(
        mut self,
        source: Option<crate::domain_computation::primary_graph::WorthQueryPendingSourceExpectation<<B::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<S>>::Query>>,
    ) -> Self {
        self.pending_source = source;
        self
    }
    pub(super) fn bind_source(
        &self,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationIdempotencyBinding {
        self.pending_source.as_ref().map_or(binding, |source| {
            source.bound_source().bind_idempotency(binding)
        })
    }
    pub(super) fn consume_source(
        &mut self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if let Some(source) = self.pending_source.take() {
            source
                .consume_into(context.application, context.phase, &mut self.operation)
                .map_err(|cause| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::ForeignSource,
                        format!("authored stage source expectation: {cause:?}"),
                    )
                })?;
        }
        Ok(())
    }
}

pub(super) struct RetainedAuthoredAdmission<S: ApplicationSchema, B: ApplicationMutationBinding<S>> {
    pub(super) operation: AuthoredEditStageOperation<S, B>,
    pub(super) pending_source: Option<crate::domain_computation::primary_graph::WorthQueryPendingSourceExpectation<<B::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<S>>::Query>>,
}

pub struct CapturedAuthoredEditStage<S> {
    pub(in crate::domain_computation::primary_graph) executor:
        Arc<dyn AuthoredEditStageExecutor<S>>,
    pub(in crate::domain_computation::primary_graph) identity: ApplicationEditStageIdentity,
    pub(in crate::domain_computation::primary_graph) before:
        worth_runtime_world::facade::ProductBranchObservation,
    pub(in crate::domain_computation::primary_graph) source:
        worth_relational::facade::identity::EntityId,
    pub(in crate::domain_computation::primary_graph) binding: &'static str,
    pub(in crate::domain_computation::primary_graph::application_contribution) queries:
        Vec<DeclaredQueryRead>,
    pub(in crate::domain_computation::primary_graph::application_contribution) schema:
        worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph::application_contribution) producers:
        Vec<ProducerDependencyScopes>,
    pub(in crate::domain_computation::primary_graph::application_contribution) producer_connections:
        Vec<(std::any::TypeId, std::any::TypeId)>,
    pub(in crate::domain_computation::primary_graph::application_contribution) producer_barrier:
        Vec<String>,
    pub(in crate::domain_computation::primary_graph::application_contribution) required: Option<
        Arc<
            dyn Fn(
                    &worth_execution::ExecutionRequest<'_, '_>,
                )
                    -> super::super::producer::synchronous_outputs::CapturedSynchronousOutputs
                + Send
                + Sync,
        >,
    >,
    pub(in crate::domain_computation::primary_graph) contracts:
        worth_query_installation::facade::WorthQueryOperationReadTouchOverlapIndex,
    pub(in crate::domain_computation::primary_graph::application_contribution) ordered: bool,
}

impl<S: 'static> From<CapturedAuthoredEditStage<S>>
    for worth_query_execution_attempt::facade::ports::OpaqueAuthoredEditStage<S>
{
    fn from(stage: CapturedAuthoredEditStage<S>) -> Self {
        Self::new(stage)
    }
}
impl<S> Clone for CapturedAuthoredEditStage<S> {
    fn clone(&self) -> Self {
        Self {
            executor: self.executor.clone(),
            before: self.before.clone(),
            identity: self.identity,
            source: self.source,
            binding: self.binding,
            contracts: self.contracts.clone(),
            queries: self.queries.clone(),
            schema: self.schema.clone(),
            producers: self.producers.clone(),
            producer_connections: self.producer_connections.clone(),
            producer_barrier: self.producer_barrier.clone(),
            required: self.required.clone(),
            ordered: self.ordered,
        }
    }
}
// These are execution mechanics, never an authority trait supplied by a consumer.
pub(in crate::domain_computation::primary_graph) trait AuthoredEditStageExecutor<S>:
    Send + Sync
{
    fn order_authored(&self);
    fn bind_correction(
        &self,
        intent: crate::domain_computation::primary_graph::WorthQueryCorrectionIntent,
    );
    fn plan(
        &self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
        unseen: &super::UnseenAuthoredEnvelope<'_>,
    ) -> Result<(), WorthQueryOutputDemandDenial>;
    fn footprint(&self) -> Option<super::planning::Footprint>;
    fn discard_plan(&self);
    fn prefix_candidate(
        &self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
        unseen: &super::UnseenAuthoredEnvelope<'_>,
    ) -> Result<Box<dyn Any + Send>, WorthQueryOutputDemandDenial>;
    fn product(
        &self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
    ) -> Result<crate::basis::WorthQueryProductBranchLease, super::envelope::Denial>;
    fn govern_envelope(
        &self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
        intent: worth_query_declaration::facade::application_operation::ApplicationEditIntentIdentity,
        next: &mut dyn FnMut(
            WorthQueryApplicationIdempotencyBinding,
        )
            -> Result<super::envelope::Resolution, super::envelope::Denial>,
    ) -> Result<super::envelope::Resolution, super::envelope::Denial>;
    fn prepare<'run>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &AuthoredEditStageContext<'_, 'run, '_, S>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
        before: &crate::basis::WorthQueryProductBranchLease,
        before_pin: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        participants: &dyn crate::domain_computation::primary_graph::provider::AppliedOutputParticipants,
    ) -> Result<InspectedAuthoredApplication<'run>, WorthQueryOutputDemandDenial>;
    fn prepare_commit<'run>(
        &self,
        writer: InspectedCoupledWriter<'run>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
    ) -> Result<CommitPreparedCoupledWriter<'run>, WorthQueryApplicationCommitOutcome>;
    fn authorize(
        &self,
        writer: &CommitPreparedCoupledWriter<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>;
    fn complete(
        &self,
        writer: CommitPreparedCoupledWriter<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
        unpublished: Option<&crate::domain_computation::WorthQueryProductUnpublishedApplication>,
    ) -> WorthQueryApplicationCommitOutcome;
}

type Admit<S, B> = dyn Fn(
        &AuthoredEditStageContext<'_, '_, '_, S>,
    ) -> Result<AuthoredEditStageAdmission<S, B>, WorthQueryOutputDemandDenial>
    + Send
    + Sync;
pub(super) struct TypedStage<S: ApplicationSchema, B: ApplicationMutationBinding<S>> {
    pub(super) original_admission: std::sync::Mutex<Option<RetainedAuthoredAdmission<S, B>>>,
    pub(super) principal_value: worth_foundational::facade::AspectValue,
    pub(super) ordered: std::sync::atomic::AtomicBool,
    pub(super) correction: std::sync::Mutex<
        Option<crate::domain_computation::primary_graph::WorthQueryCorrectionIntent>,
    >,
    pub(super) planning: std::sync::Mutex<super::planning::StagePlanning<S, B>>,
    pub(super) before: worth_runtime_world::facade::ProductBranchObservation,
    pub(super) key: Arc<ApplicationEditEnvelopeKey<B::IdempotencyKey>>,
    pub(super) input: ApplicationEncodedInput<B::InputBinding>,
    pub(super) ordinal: usize,
    pub(super) source: worth_relational::facade::identity::EntityId,
    pub(super) principal: WorthQueryAuthenticatedExternalPrincipal<S>,
    pub(super) admit: Box<Admit<S, B>>,
    pub(super) result: Arc<std::sync::Mutex<Option<Box<dyn Any + Send>>>>,
}
impl<S: ApplicationSchema> CapturedAuthoredEditStage<S> {
    pub fn capture<B>(
        _access: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &dyn crate::ports::ApplicationRuntimeRef<S>,
        key: Arc<ApplicationEditEnvelopeKey<B::IdempotencyKey>>,
        input: ApplicationEncodedInput<B::InputBinding>,
        ordinal: usize,
        principal: WorthQueryAuthenticatedExternalPrincipal<S>,
        admit: impl Fn(
                &AuthoredEditStageContext<'_, '_, '_, S>,
            )
                -> Result<AuthoredEditStageAdmission<S, B>, WorthQueryOutputDemandDenial>
            + Send
            + Sync
            + 'static,
        result: Arc<std::sync::Mutex<Option<Box<dyn Any + Send>>>>,
    ) -> Result<Self, WorthQueryOutputDemandDenial>
    where
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
        B::IdempotencyKey: Send + Sync,
        B::Result: Send,
        B::Denial: Send,
    {
        let outer_application = application.outer_application_any();
        let application_inner = application.runtime_inner();
        let admitted = admit(&AuthoredEditStageContext {
            phase,
            application: application_inner,
            outer_application,
        })?;
        Self::capture_admitted(
            _access,
            application,
            key,
            input,
            ordinal,
            principal,
            admitted,
            admit,
            result,
        )
    }

    pub fn capture_admitted<B>(
        _access: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        application: &dyn crate::ports::ApplicationRuntimeRef<S>,
        key: Arc<ApplicationEditEnvelopeKey<B::IdempotencyKey>>,
        input: ApplicationEncodedInput<B::InputBinding>,
        ordinal: usize,
        principal: WorthQueryAuthenticatedExternalPrincipal<S>,
        admitted: AuthoredEditStageAdmission<S, B>,
        admit: impl Fn(
                &AuthoredEditStageContext<'_, '_, '_, S>,
            )
                -> Result<AuthoredEditStageAdmission<S, B>, WorthQueryOutputDemandDenial>
            + Send
            + Sync
            + 'static,
        result: Arc<std::sync::Mutex<Option<Box<dyn Any + Send>>>>,
    ) -> Result<Self, WorthQueryOutputDemandDenial>
    where
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
        B::IdempotencyKey: Send + Sync,
        B::Result: Send,
        B::Denial: Send,
    {
        let application = application.runtime_inner();
        let principal_value = <B::PrincipalIdentityBinding as worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding>::encode(&admitted.principal)
            .map_err(|cause| stage_admission_refusal::<S, B>(_access, ordinal, format!("principal identity: {cause:?}")))?;
        let source = admitted.operation.scope_entity_id();
        let before = admitted
            .operation
            .graph_work()
            .mutation_product()
            .expect("genuine mutation admission holds its C product")
            .observation()
            .clone();
        let queries = managed_reads(
            application,
            admitted
                .operation
                .allowed_graph_contract()
                .managed_index_reads(),
        )?;
        let schema = application.installed_schema.binding_identity();
        let contracts = admitted
            .operation
            .allowed_graph_contract()
            .read_touch_overlap()
            .clone();
        let identity = *key
            .stage_with_encoded_input::<S, B>(&input, ordinal)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    format!("stage {ordinal} {} identity: {cause:?}", B::IDENTITY),
                )
            })?
            .edit_stage_identity()
            .expect("canonical stage");
        Ok(Self {
            source,
            before: before.clone(),
            contracts,
            queries,
            schema,
            producers: Vec::new(),
            producer_connections: Vec::new(),
            producer_barrier: Vec::new(),
            required: None,
            ordered: false,
            identity,
            binding: B::IDENTITY,
            executor: Arc::new(TypedStage::<S, B> {
                original_admission: std::sync::Mutex::new(Some(RetainedAuthoredAdmission {
                    operation: admitted.operation,
                    pending_source: admitted.pending_source,
                })),
                principal_value,
                ordered: std::sync::atomic::AtomicBool::new(false),
                correction: std::sync::Mutex::new(None),
                planning: std::sync::Mutex::new(super::planning::StagePlanning::default()),
                before,
                key,
                input,
                ordinal,
                source,
                principal,
                admit: Box::new(admit),
                result,
            }),
        })
    }
}

impl<S> CapturedAuthoredEditStage<S> {
    pub fn with_correction_intent(
        self,
        _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        intent: crate::domain_computation::primary_graph::WorthQueryCorrectionIntent,
    ) -> Self {
        self.executor.bind_correction(intent);
        self
    }
    pub fn same_admitted_basis(&self, other: &Self) -> bool {
        self.before == other.before
    }
    pub fn subject(&self) -> super::dag::DeclaredEditStage {
        super::dag::DeclaredEditStage {
            identity: self.identity,
            binding: self.binding,
        }
    }
}
