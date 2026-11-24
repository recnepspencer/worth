use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport;
use crate::domain_computation::authorization::{
    WorthQueryInstalledAuthorizationRegistry, WorthQueryRuntimeClock,
};
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntime;
use crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry;

use super::provider::WorthQueryPrimaryGraphProvider;
use super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
    WorthQueryPrimaryGraphPublication,
};
pub use crate::domain_computation::application_aftermath::WorthQueryInboundCleanupReport;

#[path = "../application_runtime_core.rs"]
pub mod runtime_core;
pub use runtime_core::WorthQueryApplicationRuntimeCore;

pub mod candidate_input_observation;
pub mod certification_controls;
pub mod certification_cost;
pub mod certification_managed_index;
pub mod derived_relation;
pub mod durable_admission;
pub mod execution_memory;
pub mod external_dispatch_attempt;
pub mod graph_participation;
pub mod home_retirement;
pub mod home_retirement_denial;
pub mod inbound_occurrence;
pub mod inbound_publication;
pub mod installed_package_export;
pub mod managed_cleanup;
pub mod mutation_partition;
pub mod operation_control;
pub mod operation_input_capture;
pub mod oracle_observation;
pub mod provider_session_preparation;
pub mod recovery_custody;
pub mod storage_retention;
pub mod teardown;

pub use certification_cost::{
    WorthQueryCertificationApplicationWork, WorthQueryCertificationCostObservation,
    WorthQueryCertificationCostRuntimeExt, WorthQueryCertificationCostScope,
    WorthQueryCertificationWorldHistory, WorthQueryCertificationWorldRetention,
};
pub use derived_relation::WorthQueryRuleReadableCollectionLifecycleForTest;
pub use execution_memory::WorthQueryExecutionLeaseDenial;
pub use external_dispatch_attempt::{
    WorthQueryExternalDispatchAdmissionDenial, WorthQueryExternalDispatchAttemptOrdinal,
    WorthQueryTerminalEffectRefusal,
};
pub use home_retirement_denial::{WorthQueryHomeRetirementDenial, WorthQueryHomeRetirementOwner};
pub use inbound_occurrence::{
    WorthQueryAdmittedInboundOccurrence, WorthQueryAuthenticatedInboundOccurrence,
    WorthQueryCorrelatedInboundOccurrence, WorthQueryInboundAdmissionDenial,
    WorthQueryInboundAuthenticatedPermanentDenial, WorthQueryInboundCostObservation,
    WorthQueryInboundIndexRepairDenial, WorthQueryInboundMaintenanceReport,
    WorthQueryInboundPendingReason, WorthQueryInboundPermanentDenialKind, WorthQueryInboundReceipt,
    WorthQueryInboundReceiptPosture, WorthQueryInboundSourceControlDenial,
    WorthQueryInboundSourcePosture, WorthQueryInboundTerminalObservation,
    WorthQueryInboundVerifierHandle, WorthQueryInboundVerifierInstallationDenial,
    WorthQueryInstalledTransportCompletionBinding,
};
pub use inbound_publication::{
    InstalledTransportCompletion, InstalledTransportPendingReason, InstalledTransportResumeOutcome,
    PerformedInstalledTransportCompletion, WorthQueryInboundPublicationDenial,
    WorthQueryInboundPublicationOutcome, WorthQueryPerformedInboundCompletion,
    WorthQueryRecordedCompletionDenial, WorthQueryUnpublishedInboundCompletion,
};
pub use operation_control::{
    WorthQueryApplicationAttemptOperationControl, WorthQuerySourcePublicationHeadForTest,
    WorthQuerySourcePublicationImageForTest,
};
pub use operation_input_capture::WorthQueryOperationInputCaptureDenial;
pub use oracle_observation::{
    WorthQueryOracleHistoryPageForTest, WorthQueryOracleOccurrenceForTest,
    WorthQueryOracleSignalMovementForTest,
};
pub use storage_retention::{
    WorthQueryStorageRetentionDenial, WorthQueryStorageRetentionProgress,
    WorthQueryStorageRetentionStop,
};

/// Concrete state fields owned by `WorthQueryPrimaryGraphApplicationRuntimeInner<Schema>`.
pub struct WorthQueryPrimaryGraphApplicationRuntimeFields<Schema> {
    pub installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
    pub runtime: WorthQueryApplicationRuntimeCore,
    pub post_conditional_teardown: teardown::ApplicationPostConditionalTeardown,
    pub mutation_handlers: super::handler::InstalledMutationHandlerRegistry<Schema>,
    pub mutation_projection:
        super::WorthQueryApplicationInvariantProjectionAuthority<Schema>,
    pub installed_producers:
        super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>,
    pub output_teardown: teardown::ApplicationOutputTeardown,
    pub program_required_bindings: std::collections::HashSet<TypeId>,
    pub program_required_operations: std::collections::HashSet<TypeId>,
    pub workflow_guarded_operations: std::collections::HashSet<TypeId>,
    pub program_support:
        Option<super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>>,
    pub signing_owners: std::sync::Mutex<
        std::collections::BTreeMap<
            worth_query_declaration::facade::application_program::ApplicationProgramRevision,
            worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner<Schema>,
        >,
    >,
    pub installed_conditionals:
        super::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<Schema>,
    pub managed_indexes:
        super::application_query::managed_index::ManagedIndexProviderCatalog<Schema>,
    pub workflow_teardown: teardown::ApplicationWorkflowTeardown,
    #[doc(hidden)]
    pub conditional_admission_port:
        std::sync::Weak<dyn crate::ports::ConditionalDefinitionAdmissionPort>,
}

trait RuntimeInnerFieldsBacking<Schema>: Send + Sync {
    fn as_fields(&self) -> &WorthQueryPrimaryGraphApplicationRuntimeFields<Schema>;
    fn as_fields_mut(&mut self) -> &mut WorthQueryPrimaryGraphApplicationRuntimeFields<Schema>;
}

impl<Schema> RuntimeInnerFieldsBacking<Schema>
    for WorthQueryPrimaryGraphApplicationRuntimeFields<Schema>
{
    fn as_fields(&self) -> &WorthQueryPrimaryGraphApplicationRuntimeFields<Schema> {
        self
    }

    fn as_fields_mut(&mut self) -> &mut WorthQueryPrimaryGraphApplicationRuntimeFields<Schema> {
        self
    }
}

/// Core state owned by `WorthQueryPrimaryGraphApplicationRuntime<Schema>` after
/// its leading conditional-cleanup field.
pub struct WorthQueryPrimaryGraphApplicationRuntimeInner<Schema>(
    Box<dyn RuntimeInnerFieldsBacking<Schema>>,
);

impl<Schema: 'static> WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
    pub fn new(fields: WorthQueryPrimaryGraphApplicationRuntimeFields<Schema>) -> Self {
        Self(Box::new(fields))
    }
}

impl<Schema> std::ops::Deref for WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
    type Target = WorthQueryPrimaryGraphApplicationRuntimeFields<Schema>;

    fn deref(&self) -> &Self::Target {
        self.0.as_fields()
    }
}

impl<Schema> std::ops::DerefMut for WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_fields_mut()
    }
}

/// Crate-internal alias so all moved `impl WorthQueryPrimaryGraphApplicationRuntime<Schema>`
/// blocks compile directly against `WorthQueryPrimaryGraphApplicationRuntimeInner<Schema>`.
pub type WorthQueryPrimaryGraphApplicationRuntime<Schema> =
    WorthQueryPrimaryGraphApplicationRuntimeInner<Schema>;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
    pub fn operation_requires_application_program<Operation: 'static>(&self) -> bool {
        self.program_support.is_some()
            || self
                .program_required_operations
                .contains(&TypeId::of::<Operation>())
    }

    pub fn operation_requires_workflow_authority<Operation: 'static>(&self) -> bool {
        self.workflow_guarded_operations
            .contains(&TypeId::of::<Operation>())
    }

    pub fn requires_application_program<Binding>(&self) -> bool
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        self.program_support.is_some()
            || self
                .program_required_bindings
                .contains(&TypeId::of::<Binding>())
            || self
                .program_required_operations
                .contains(&TypeId::of::<Binding::Operation>())
    }

    pub fn is_branch_adoption_original(
        &self,
        effects: &worth_runtime_world::facade::WorldRecoveredUnpublishedEffects,
    ) -> Result<bool, super::query_obligations::QueryObligationDenial> {
        self.installed_program_support()
            .map(|support| support.lifecycle().is_adoption_original(effects))
            .transpose()
            .map(|adoption| adoption.unwrap_or(false))
    }

    pub fn installed_program_support(
        &self,
    ) -> Option<&super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        self.program_support.as_ref()
    }

    #[inline]
    pub fn authentication_is_expired(&self, valid_until: std::time::Instant) -> bool {
        self.runtime.authentication_is_expired(valid_until)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntimeInner<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn producer<Binding>(&self) -> Option<Arc<Binding::Provider>>
    where
        Binding: super::application_contribution::WorthQueryApplicationProducerBinding<Schema>,
    {
        self.installed_producers.provider::<Binding>()
    }

    pub fn current_world(&self) -> crate::basis::WorthQueryProductBranch {
        self.runtime.product_runtime.root_product_branch()
    }

    pub fn installed_schema(&self) -> &WorthQueryInstalledApplicationSchema<Schema> {
        &self.installed_schema
    }

    #[inline]
    pub fn publication(&self) -> &WorthQueryPrimaryGraphPublication {
        self.runtime.publication()
    }

    #[inline]
    pub fn checkpoint_restore_work(
        &self,
    ) -> Option<worth_relational::facade::durability::CheckpointRestoreWork> {
        self.runtime.checkpoint_restore_work()
    }

    #[inline]
    pub fn granular_invalidation_installation(
        &self,
    ) -> super::WorthQueryGranularInvalidationInstallation {
        self.runtime.granular_invalidation_installation()
    }

    #[inline]
    pub fn application_query_basis_observer(
        &self,
    ) -> super::application_query::WorthQueryApplicationBasisObserver {
        self.runtime.application_query_basis_observer()
    }

    #[inline]
    pub fn capability_plan_compilation_evidence(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryCapabilityPlanCompilationEvidence {
        self.runtime.capability_plan_compilation_evidence()
    }

    #[doc(hidden)]
    #[inline]
    pub fn workflow_compilation_reuse_counters(
        &self,
    ) -> super::WorthQueryWorkflowCompilationReuseCounters {
        self.runtime.workflow_compilation_reuse_counters()
    }

    #[doc(hidden)]
    #[inline]
    pub fn workflow_instance_progress_counters(
        &self,
    ) -> super::WorthQueryWorkflowInstanceProgressCounters {
        self.runtime.workflow_instance_progress_counters()
    }

    #[inline]
    pub fn graph_work_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.runtime.graph_work_resource_support()
    }

    #[inline]
    pub fn graph_work_resource_support_ref(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
{
        self.runtime.graph_work_resource_support_ref()
    }

    #[inline]
    pub fn graph_work_provider_identity(&self) -> &str {
        self.runtime.graph_work_provider_identity()
    }

    #[inline]
    pub fn close_live_delivery(&self) -> WorthQueryApplicationLiveDeliveryCloseReceipt {
        self.runtime.close_live_delivery()
    }
}

/// Receipt from closing this runtime's live delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationLiveDeliveryCloseReceipt {
    pub(crate) remaining_live_consumers: usize,
}

impl WorthQueryApplicationLiveDeliveryCloseReceipt {
    pub const fn remaining_live_consumers(self) -> usize {
        self.remaining_live_consumers
    }

    pub const fn owner_terminal(self) -> bool {
        self.remaining_live_consumers == 0
    }
}
