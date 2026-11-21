//! Upward boundary ports for `worth-query-execution-attempt`.

use std::any::Any;
use std::sync::Arc;

use worth_execution::{ExecutionArray, ExecutionRequest};
use worth_relational::facade::{
    branch::{DeletedRelationalBranch, RelationalBranchIdentity},
    history::BranchId,
    mvcc::PreparedRelationalChangeSummary,
    runtime::{
        ManagedIndexSourceContextFactory, ManagedIndexSourcePreparationDenial,
        ManagedIndexSourcePreparationMetadata, RelationalAppliedCandidateReadIdentity,
        RelationalPinnedRead, RelationalRuntime,
    },
    snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause,
};
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchObservation,
};

use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact as Fact,
    },
    live_delivery::WorthQueryLivePredicatePreparation,
    source_facts::SourceFactsAllocation,
    WorthQueryCommittedProductPublication,
};

/// Upper-layer applied-read support checked during coupled candidate preparation.
pub trait CandidateReadSupportPort: Send + Sync {
    fn subject(&self) -> &str;
    fn pin(&self) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;
    fn changed_fact<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        earlier: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        image: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<Option<&'a Fact>, WorthQueryApplicationAttemptDenial>;
    fn has_route(
        &self,
        _meaning: &worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    ) -> bool {
        false
    }
    fn read(&self) -> Option<crate::domain_computation::primary_graph::provider::ParentRead> {
        None
    }
}

/// Issued by the registration owner; equal probes retain distinct exact slots.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WatchSlot(pub u64);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PredicateNoticeDisposition {
    Refresh,
    Unavailable,
}

pub struct PreparedPredicateNotices {
    slots: ExecutionArray<(WatchSlot, PredicateNoticeDisposition)>,
    _allocation: Arc<SourceFactsAllocation>,
}

impl PreparedPredicateNotices {
    pub fn new(
        slots: ExecutionArray<(WatchSlot, PredicateNoticeDisposition)>,
        allocation: Arc<SourceFactsAllocation>,
    ) -> Self {
        Self {
            slots,
            _allocation: allocation,
        }
    }

    pub fn disposition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        slot: WatchSlot,
    ) -> Result<Option<PredicateNoticeDisposition>, RelationalExecutionDenialCause> {
        let slots = self.slots.elements();
        let count = slots
            .len()
            .checked_add(1)
            .ok_or(RelationalExecutionDenialCause::WorkCounterOverflow)?;
        SourceFactsAllocation::work(execution, u64::from(usize::BITS - count.leading_zeros()))?;
        Ok(slots
            .binary_search_by_key(&slot, |(slot, _)| *slot)
            .ok()
            .map(|index| slots[index].1))
    }
}

pub enum PreparedPredicatePublication {
    Matched(Arc<PreparedPredicateNotices>),
    Untracked,
}

pub trait PredicatePreparationOwnerPort: Send + Sync + Any {
    fn publication_for(
        &self,
        before: &ProductBranchObservation,
        publication: &WorthQueryCommittedProductPublication,
    ) -> PreparedPredicatePublication;
}

/// Opaque predicate-preparation custody held by commit preparation until publication or retirement.
pub struct PredicatePreparationOwner(Box<dyn PredicatePreparationOwnerPort>);

impl PredicatePreparationOwner {
    pub fn new(owner: Box<dyn PredicatePreparationOwnerPort>) -> Self {
        Self(owner)
    }

    pub fn publication_for(
        &self,
        before: &ProductBranchObservation,
        publication: &WorthQueryCommittedProductPublication,
    ) -> PreparedPredicatePublication {
        self.0.publication_for(before, publication)
    }
}

/// Created outside Native/registry gates, filled only when exact custody retires.
#[must_use]
#[derive(Default)]
pub struct PredicatePreparationRetirement {
    owner: Option<PredicatePreparationOwner>,
}

impl PredicatePreparationRetirement {
    pub fn retire(&mut self, owner: &mut Option<PredicatePreparationOwner>) {
        if owner.is_some() {
            assert!(
                self.owner.is_none(),
                "one operation retires one actual predicate preparation"
            );
            self.owner = owner.take();
        }
    }
}

pub trait PredicatePreparationFactoryPort: ManagedIndexSourceContextFactory + Send + Sync {
    fn as_context_factory(&self) -> &dyn ManagedIndexSourceContextFactory;
    fn promote(
        self: Box<Self>,
        execution: &ExecutionRequest<'_, '_>,
        accepted: ManagedIndexSourcePreparationMetadata<'_>,
    ) -> Result<PredicatePreparationOwner, ManagedIndexSourcePreparationDenial>;
}

pub struct ViewPublicationBasis<'a> {
    pub before: &'a CompositeCommitIdentity,
    pub relational_branch: &'a BranchId,
    pub product_branch: &'a ProductBranchIdentity,
    pub incarnation: ProductBranchIncarnation,
}

pub trait PreparedManagedViewPublicationPort: Send {
    fn apply(self: Box<Self>, after: &CompositeCommitIdentity);
    fn apply_cold(self: Box<Self>, after: &CompositeCommitIdentity);
}

/// Upper-layer derived-view execution port implemented by `ManagedDerivedViewRegistry`.
pub trait ManagedDerivedViewExecutionPort: Send + Sync {
    fn has_predicate_routing(&self) -> bool;
    fn begin_predicate_preparation(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
        fence: WorthQueryLivePredicatePreparation,
    ) -> Result<Box<dyn PredicatePreparationFactoryPort>, RelationalExecutionDenialCause>;
    fn manages_native_sources(&self, identity: &RelationalBranchIdentity) -> bool;
    fn retire_source_branch(&self, deleted: &DeletedRelationalBranch);
    fn prepare_view_publication(
        &self,
        basis: ViewPublicationBasis<'_>,
        summary: Option<&PreparedRelationalChangeSummary>,
        max_view_changes: usize,
        max_work_units: usize,
    ) -> Option<(Box<dyn PreparedManagedViewPublicationPort>, bool)>;
}

/// Upper-layer installed program record behind `OriginalProgramMeaning`.
pub trait SupportedProgramRecordPort: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn action_inventory(
        &self,
    ) -> worth_query_installation::facade::WorthQueryInstalledActionInventory<'_>;
    fn revision(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationProgramRevision;
    fn declares_binding(&self, binding: std::any::TypeId) -> bool;
}

/// Opaque upper-layer `CapturedAuthoredEditStage<Schema>` carried by `WorthQueryApplicationEffectProgram`.
/// Parameterized by `Schema` with a private backing box so stages cannot cross schemas
/// or be forged from Host.
pub struct OpaqueAuthoredEditStage<Schema> {
    inner: Box<dyn Any + Send + Sync>,
    _schema: std::marker::PhantomData<fn() -> Schema>,
}

impl<Schema> OpaqueAuthoredEditStage<Schema> {
    pub fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self {
            inner: Box::new(value),
            _schema: std::marker::PhantomData,
        }
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.inner.downcast_ref::<T>()
    }

    pub fn downcast<T: Any>(self) -> Result<Box<T>, Self> {
        self.inner.downcast::<T>().map_err(|inner| Self {
            inner,
            _schema: std::marker::PhantomData,
        })
    }
}

pub enum AuthoredCorrectionCandidateDenial {
    Identity(&'static str),
    Interrupted(
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    ),
}

/// Upper-layer prepared correction source port behind `WorthQueryPendingAftermathCausality`.
pub trait PreparedCorrectionSourcePort: std::fmt::Debug + Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn eq_source(&self, other: &dyn PreparedCorrectionSourcePort) -> bool;
    fn lineage_binding(
        &self,
    ) -> &crate::domain_computation::application_aftermath::WorthQueryCorrectionLineageBinding;
    fn target_parent(&self) -> &worth_relational::facade::history::RelationalCommitReceipt;
    fn target_branch(&self) -> &worth_relational::facade::history::BranchId;
    fn source_world(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity;
    fn source_commit(&self) -> &worth_relational::facade::history::RelationalCommitReceipt;
    fn intent_causal_key(&self) -> String;
    fn authored_declaration(
        &self,
    ) -> Option<&worth_query_declaration::facade::application_aftermath::DeclaredAuthoredCorrection>;
    fn authored_retirement_anchor(&self) -> Option<worth_relational::facade::identity::EntityId>;
    fn authored_retirement_co_anchors(&self) -> Vec<worth_relational::facade::identity::EntityId>;
    fn authored_restoration_anchor(
        &self,
        roles: &[(&str, worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture, &str, &worth_relational::facade::transactions::EntityReference)],
        batch: &worth_relational::facade::transactions::WorkerIntentBatch,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        Option<(
            worth_relational::facade::identity::EntityId,
            worth_relational::facade::transactions::CreatedEntityRef,
        )>,
        AuthoredCorrectionCandidateDenial,
    >;
    fn authored_restoration_co_anchors(
        &self,
        roles: &[(&str, worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture, &str, &worth_relational::facade::transactions::EntityReference)],
    ) -> Vec<(
        worth_relational::facade::identity::EntityId,
        worth_relational::facade::transactions::CreatedEntityRef,
    )>;
    fn matches_admission_basis(
        &self,
        runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        principal: &crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        target: Option<&crate::basis::WorthQueryProductBranchReadIdentity>,
    ) -> bool;
    fn admit_home_receiver(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), worth_proof::HomeIncarnationDenial>;
    fn admit_receiver(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial>;
    fn material_pin(&self) -> Option<worth_foundational::facade::CorrectionMaterialPinIdentity>;
    fn validate_authored_candidate(
        &self,
        roles: &[(&str, worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture, &str, &worth_relational::facade::transactions::EntityReference)],
        batch: &worth_relational::facade::transactions::WorkerIntentBatch,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<(), AuthoredCorrectionCandidateDenial>;
    fn validate_workflow_candidate(
        &self,
        batch: &worth_relational::facade::transactions::WorkerIntentBatch,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>;
    fn validate_relation_inverse(
        &self,
        candidate: &worth_relational::facade::mvcc::ValidatedRelationalProposal,
        execution: &ExecutionRequest<'_, '_>,
        layout: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    ) -> Result<(), crate::domain_computation::WorthQueryProviderSessionFailure>;
    fn changed_field_capture(
        &self,
    ) -> Result<
        Option<(
            worth_query_installation::facade::InstalledPreImageDemand,
            worth_relational::facade::identity::EntityId,
        )>,
        (),
    >;
    fn capture_coupled_outputs(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Option<crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemands>;
    fn content(&self) -> &worth_relational::facade::content::RelationalContentAttachments;
}

/// Borrowed upper-layer history cutover handle behind `HistoryProducerCutover<'call>`.
pub trait HistoryProducerCutoverPort {
    fn uses_owned_model_graph(&self) -> bool;
    fn current(&self) -> &ProductBranchObservation;
    fn perform(
        &self,
        cutover: worth_runtime_world::facade::PreparedWorldRelationalCutover<'_, '_, '_, '_>,
        expected_target: &ProductBranchObservation,
    ) -> Result<(), ()>;
}

mod opaque_synchronous_outputs;
pub use opaque_synchronous_outputs::OpaqueSynchronousOutputs;

/// Upper-layer workflow vocabulary read port used by `approval_decision`.
pub trait WorkflowVocabularyPort<Schema, Spec>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Spec: worth_query_declaration::facade::application_program::ApplicationWorkflowSpec<
        Schema = Schema,
    >,
{
    fn workflow_spec_at_revision(
        &self,
        revision: &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    ) -> Option<
        &worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    >;
    fn authentication_owner(
        &self,
    ) -> &worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner<Schema>;
}

pub mod admitted_operation_port;
pub mod authenticated_and_recovery_port;
pub mod commit_and_retirement;
mod complete_read_set_methods_port;
pub mod complete_read_set_port;
mod complete_read_set_workflow_impl;
mod effect_program_builder_port;
mod effect_program_methods_port;
pub mod effect_program_port;
mod history_and_output_source_port;
mod projected_read_set_port;
pub mod read_attempt_admission;
mod recovery_handle_binding_port;
pub mod runtime_ports;
mod specialized_program_port;
pub use {
    admitted_operation_port::AdmittedOperationExecutionPort,
    authenticated_and_recovery_port::*,
    commit_and_retirement::{CommitAuthorizationRuntimePort, WorkflowDefinitionRetirementPort},
    complete_read_set_methods_port::*,
    complete_read_set_port::{CompleteReadSetInternalPort, CompleteReadSetParts},
    effect_program_builder_port::*,
    effect_program_methods_port::*,
    effect_program_port::{
        EffectProgramBuilderInternalPort, EffectProgramInternalPort, EffectProgramParts,
    },
    history_and_output_source_port::*,
    projected_read_set_port::*,
    read_attempt_admission::{
        ApplicationReadAttemptAdmissionPort, WorthQueryReadAttemptRuntimeContext,
    },
    recovery_handle_binding_port::*,
    runtime_ports::*,
    specialized_program_port::*,
};
