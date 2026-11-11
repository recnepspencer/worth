use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport;
use crate::domain_computation::authorization::WorthQueryRuntimeClock;
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntime;
use crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry;

use super::provider::WorthQueryPrimaryGraphProvider;
use super::{
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind, WorthQueryPrimaryGraphPublication,
};
pub use crate::domain_computation::application_aftermath::WorthQueryInboundCleanupReport;
use crate::domain_computation::authorization::WorthQueryInstalledAuthorizationRegistry;

#[path = "application_runtime_core.rs"]
pub(crate) mod runtime_core;
pub(in crate::domain_computation) use runtime_core::WorthQueryApplicationRuntimeCore;

mod certification_controls;
mod certification_cost;
mod conditional_cleanup;
mod home_retirement;
mod oracle_observation;
mod provider_session_preparation;
pub(in crate::domain_computation::primary_graph) use home_retirement::RetiringPrimaryGraphRuntime;
pub use home_retirement::{WorthQueryHomeRetirementDenial, WorthQueryHomeRetirementOwner};
pub use oracle_observation::{
    WorthQueryOracleHistoryPageForTest, WorthQueryOracleOccurrenceForTest,
    WorthQueryOracleSignalMovementForTest,
};
mod durable_admission;
mod execution_memory;
pub(crate) mod external_dispatch_attempt;
mod mutation_partition;
mod teardown;
pub use execution_memory::WorthQueryExecutionLeaseDenial;
pub(crate) mod inbound_occurrence;

pub(crate) mod inbound_publication;
pub use certification_cost::{
    WorthQueryCertificationApplicationWork, WorthQueryCertificationCostObservation,
    WorthQueryCertificationCostRuntimeExt, WorthQueryCertificationCostScope,
    WorthQueryCertificationWorldHistory, WorthQueryCertificationWorldRetention,
};
pub(in crate::domain_computation) use inbound_occurrence::WorthQueryInstalledTransportCompletionBinding;
pub use inbound_occurrence::{
    WorthQueryAdmittedInboundOccurrence, WorthQueryAuthenticatedInboundOccurrence,
    WorthQueryCorrelatedInboundOccurrence, WorthQueryInboundAdmissionDenial,
    WorthQueryInboundAuthenticatedPermanentDenial, WorthQueryInboundCostObservation,
    WorthQueryInboundIndexRepairDenial, WorthQueryInboundMaintenanceReport,
    WorthQueryInboundPendingReason, WorthQueryInboundPermanentDenialKind, WorthQueryInboundReceipt,
    WorthQueryInboundReceiptPosture, WorthQueryInboundSourceControlDenial,
    WorthQueryInboundSourcePosture, WorthQueryInboundTerminalObservation,
    WorthQueryInboundVerifierHandle, WorthQueryInboundVerifierInstallationDenial,
};
pub use inbound_publication::PerformedInstalledTransportCompletion;
pub use inbound_publication::WorthQueryPerformedInboundCompletion;
pub use inbound_publication::WorthQueryUnpublishedInboundCompletion;
pub(in crate::domain_computation::primary_graph) use inbound_publication::{
    InstalledTransportCompletion, InstalledTransportPendingReason, InstalledTransportResumeOutcome,
};
pub(in crate::domain_computation) use inbound_publication::{
    WorthQueryInboundPublicationDenial, WorthQueryInboundPublicationOutcome,
};
mod derived_relation;
mod graph_participation;
pub use derived_relation::WorthQueryRuleReadableCollectionLifecycleForTest;
pub(in crate::domain_computation::primary_graph) mod installation;
pub(crate) mod operation_control;
mod publication_entry;
mod storage_retention;
pub(in crate::domain_computation::primary_graph) use operation_control::WorthQueryApplicationAttemptOperationControl;
pub use storage_retention::{
    WorthQueryStorageRetentionDenial, WorthQueryStorageRetentionProgress,
    WorthQueryStorageRetentionStop,
};

pub(in crate::domain_computation) use external_dispatch_attempt::{
    WorthQueryExternalDispatchAdmissionDenial, WorthQueryExternalDispatchAttemptOrdinal,
    WorthQueryTerminalEffectRefusal,
};

/// Purpose-scoped application runtime published from one typed primary graph.
/// Publishing consumes the raw execution root and its installation authority.
/// The resulting value exposes principal admission and installed-handler
/// candidate construction, but no provider-session, commit, publication,
/// workflow, live, or replay authority.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
///
/// fn cannot_extract_execution_authority<Schema: ApplicationSchema>(
///     application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
/// ) {
///     let _ = application.installed_packages();
/// }
/// ```
///
/// A bare application runtime cannot resolve a principal through an implicit
/// current product.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
///
/// fn bare_runtime_cannot_resolve_principal<Schema: ApplicationSchema>(
///     application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
/// ) {
///     let _ = application.resolve_authenticated_principal();
/// }
/// ```
///
/// A bare application runtime cannot admit an ordinary query through an
/// implicit current product.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
///
/// fn bare_runtime_cannot_admit_query<Schema: ApplicationSchema>(
///     application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
/// ) {
///     let _ = application.admit_application_query();
/// }
/// ```
pub struct WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    // Cleanup custody precedes the core and its dependent typed registries.
    pub(super) conditional_cleanup: conditional_cleanup::ConditionalRuntimeCleanup<Schema>,
    pub(in crate::domain_computation) installed_schema:
        WorthQueryInstalledApplicationSchema<Schema>,
    pub(in crate::domain_computation) runtime: WorthQueryApplicationRuntimeCore,
    post_conditional_teardown: teardown::ApplicationPostConditionalTeardown,
    pub(super) mutation_handlers: super::handler::InstalledMutationHandlerRegistry<Schema>,
    pub(super) mutation_projection:
        super::WorthQueryApplicationInvariantProjectionAuthority<Schema>,
    pub(super) installed_producers:
        super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>,
    output_teardown: teardown::ApplicationOutputTeardown,
    pub(super) program_required_bindings: std::collections::HashSet<TypeId>,
    pub(super) program_required_operations: std::collections::HashSet<TypeId>,
    pub(super) workflow_guarded_operations: std::collections::HashSet<TypeId>,
    pub(super) program_support:
        Option<super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>>,
    pub(super) signing_owners: std::sync::Mutex<std::collections::BTreeMap<worth_query_declaration::facade::application_program::ApplicationProgramRevision, worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner<Schema>>>,
    pub(super) installed_conditionals:
        super::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<Schema>,
    pub(super) managed_indexes: super::application_query::managed_index::ManagedIndexProviderCatalog<Schema>,
    workflow_teardown: teardown::ApplicationWorkflowTeardown,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn operation_requires_application_program<
        Operation: 'static,
    >(
        &self,
    ) -> bool {
        self.program_support.is_some()
            || self
                .program_required_operations
                .contains(&TypeId::of::<Operation>())
    }

    pub(in crate::domain_computation) fn operation_requires_workflow_authority<
        Operation: 'static,
    >(
        &self,
    ) -> bool {
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

    /// The immutable program support this host admitted at installation, when
    /// it admitted any.
    pub(super) fn installed_program_support(
        &self,
    ) -> Option<&super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        self.program_support.as_ref()
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[inline]
    pub(in crate::domain_computation) fn authentication_is_expired(
        &self,
        valid_until: std::time::Instant,
    ) -> bool {
        self.runtime.authentication_is_expired(valid_until)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn producer<Binding>(&self) -> Option<Arc<Binding::Provider>>
    where
        Binding: super::application_contribution::WorthQueryApplicationProducerBinding<Schema>,
    {
        self.installed_producers.provider::<Binding>()
    }

    pub fn conditional<Binding>(&self) -> Option<Arc<Binding::Installed>>
    where
        Binding: super::application_contribution::WorthQueryApplicationConditionalBinding<Schema>,
    {
        self.installed_conditionals.binding::<Binding>()
    }

    pub fn installed_schema(&self) -> &WorthQueryInstalledApplicationSchema<Schema> {
        &self.installed_schema
    }

    #[inline]
    pub fn publication(&self) -> &WorthQueryPrimaryGraphPublication {
        self.runtime.publication()
    }

    /// Native checkpoint readmission work for this installation, if it was
    /// restored. This is diagnostic only and grants no recovered authority.
    #[inline]
    pub fn checkpoint_restore_work(
        &self,
    ) -> Option<worth_relational::facade::durability::CheckpointRestoreWork> {
        self.runtime.checkpoint_restore_work()
    }

    /// Retains the opaque installation needed to bind Query maintenance to
    /// this exact primary application runtime.
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
    pub(in crate::domain_computation) fn graph_work_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.runtime.graph_work_resource_support()
    }

    #[inline]
    pub(in crate::domain_computation) fn graph_work_resource_support_ref(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
{
        self.runtime.graph_work_resource_support_ref()
    }

    #[inline]
    pub(in crate::domain_computation) fn graph_work_provider_identity(&self) -> &str {
        self.runtime.graph_work_provider_identity()
    }

    /// Closes ordinary live delivery without closing the authoritative graph.
    /// Later commits retain their compact idempotency causality but no longer
    /// enter this runtime's delivery ring.
    #[inline]
    pub fn close_live_delivery(&self) -> WorthQueryApplicationLiveDeliveryCloseReceipt {
        self.runtime.close_live_delivery()
    }
}

/// Receipt from closing this runtime's live delivery.
///
/// Later commits no longer enter live delivery. `owner_terminal()` is true once
/// no live consumers remain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationLiveDeliveryCloseReceipt {
    remaining_live_consumers: usize,
}

impl WorthQueryApplicationLiveDeliveryCloseReceipt {
    pub const fn remaining_live_consumers(self) -> usize {
        self.remaining_live_consumers
    }

    pub const fn owner_terminal(self) -> bool {
        self.remaining_live_consumers == 0
    }
}

mod candidate_input_observation;

mod operation_input_capture;
pub use operation_input_capture::WorthQueryOperationInputCaptureDenial;

pub(in crate::domain_computation::primary_graph) use operation_control::{
    WorthQuerySourcePublicationHeadForTest, WorthQuerySourcePublicationImageForTest,
};

mod court_doors_for_test;
#[cfg(test)]
#[path = "application_runtime_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

#[cfg(test)]
fn collect_owned_inline_type_sizes(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryPrimaryGraphApplicationRuntime<()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_runtime.rs", "WorthQueryPrimaryGraphApplicationRuntime", 128 * 1024);
        owner.field::<conditional_cleanup::ConditionalRuntimeCleanup<()>>("conditional_cleanup");
        owner.field::<WorthQueryInstalledApplicationSchema<()>>("installed_schema");
        owner.field::<WorthQueryApplicationRuntimeCore>("runtime");
        owner.field::<teardown::ApplicationPostConditionalTeardown>("post_conditional_teardown");
        owner.field::<super::handler::InstalledMutationHandlerRegistry<()>>("mutation_handlers");
        owner.field::<super::WorthQueryApplicationInvariantProjectionAuthority<()>>(
            "mutation_projection",
        );
        owner.field::<super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<()>>("installed_producers");
        owner.field::<teardown::ApplicationOutputTeardown>("output_teardown");
        owner.field::<std::collections::HashSet<TypeId>>("program_required_bindings");
        owner.field::<std::collections::HashSet<TypeId>>("program_required_operations");
        owner.field::<std::collections::HashSet<TypeId>>("workflow_guarded_operations");
        owner.field::<Option<super::program_occurrence::WorthQueryInstalledProgramSupport<()>>>(
            "program_support",
        );
        owner.field::<std::sync::Mutex<std::collections::BTreeMap<worth_query_declaration::facade::application_program::ApplicationProgramRevision, worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner<()>>>>("signing_owners");
        owner.field::<super::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<()>>("installed_conditionals");
        owner.field::<super::application_query::managed_index::ManagedIndexProviderCatalog<()>>(
            "managed_indexes",
        );
        owner.field::<teardown::ApplicationWorkflowTeardown>("workflow_teardown");
    }
}
