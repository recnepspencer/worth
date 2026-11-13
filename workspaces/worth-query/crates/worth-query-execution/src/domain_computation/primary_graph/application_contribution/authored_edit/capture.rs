//! Nominal typed stage factories remain typed behind request-local erasure.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    CommitPreparedCoupledWriter, InspectedAuthoredApplication, InspectedCoupledWriter,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::application_operation::{
    ApplicationEditEnvelopeKey, ApplicationEncodedInput,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

pub struct AuthoredEditStageAdmission<S: ApplicationSchema, B: ApplicationMutationBinding<S>> {
    pub(super) operation: AuthoredEditStageOperation<S, B>,
    pub(super) principal: B::PrincipalIdentity,
}
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> AuthoredEditStageAdmission<S, B> {
    pub fn new(
        operation: AuthoredEditStageOperation<S, B>,
        principal: B::PrincipalIdentity,
    ) -> Self {
        Self {
            operation,
            principal,
        }
    }
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
        }
    }
}
// These are execution mechanics, never an authority trait supplied by a consumer.
pub(in crate::domain_computation::primary_graph) trait AuthoredEditStageExecutor<S>:
    Send + Sync
{
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
        _access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
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
        let admitted = admit(&AuthoredEditStageContext { phase, application })?;
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
            identity,
            binding: B::IDENTITY,
            executor: Arc::new(TypedStage::<S, B> {
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
