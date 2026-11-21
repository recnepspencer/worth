use std::sync::Arc;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use super::provider::WorthQueryPrimaryGraphProvider;
use super::{
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind, WorthQueryPrimaryGraphPublication,
};
pub use crate::domain_computation::application_aftermath::WorthQueryInboundCleanupReport;

pub use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::{
    certification_managed_index,
    external_dispatch_attempt, graph_participation, home_retirement_denial, inbound_occurrence,
    inbound_publication, mutation_partition, InstalledTransportCompletion, InstalledTransportResumeOutcome,
    PerformedInstalledTransportCompletion, WorthQueryAdmittedInboundOccurrence, WorthQueryApplicationLiveDeliveryCloseReceipt,
    WorthQueryApplicationRuntimeCore, WorthQueryAuthenticatedInboundOccurrence,
    WorthQueryCertificationApplicationWork, WorthQueryCertificationCostObservation,
    WorthQueryCertificationCostRuntimeExt, WorthQueryCertificationCostScope,
    WorthQueryCertificationWorldHistory, WorthQueryCertificationWorldRetention,
    WorthQueryCorrelatedInboundOccurrence, WorthQueryExecutionLeaseDenial, WorthQueryExternalDispatchAttemptOrdinal,
    WorthQueryHomeRetirementDenial, WorthQueryHomeRetirementOwner,
    WorthQueryInboundAdmissionDenial, WorthQueryInboundAuthenticatedPermanentDenial,
    WorthQueryInboundCostObservation, WorthQueryInboundIndexRepairDenial,
    WorthQueryInboundMaintenanceReport, WorthQueryInboundPendingReason,
    WorthQueryInboundPermanentDenialKind, WorthQueryInboundReceipt,
    WorthQueryInboundReceiptPosture, WorthQueryInboundSourceControlDenial,
    WorthQueryInboundSourcePosture, WorthQueryInboundTerminalObservation,
    WorthQueryInboundVerifierHandle, WorthQueryInboundVerifierInstallationDenial,
    WorthQueryInstalledTransportCompletionBinding, WorthQueryOperationInputCaptureDenial,
    WorthQueryOracleHistoryPageForTest, WorthQueryOracleOccurrenceForTest,
    WorthQueryOracleSignalMovementForTest, WorthQueryPerformedInboundCompletion,
    WorthQueryPrimaryGraphApplicationRuntimeInner, WorthQueryRecordedCompletionDenial,
    WorthQueryRuleReadableCollectionLifecycleForTest, WorthQueryStorageRetentionDenial,
    WorthQueryStorageRetentionProgress, WorthQueryStorageRetentionStop,
    WorthQueryTerminalEffectRefusal, WorthQueryUnpublishedInboundCompletion,
};

mod conditional_cleanup;
mod home_retirement;
pub(in crate::domain_computation::primary_graph) use home_retirement::RetiringPrimaryGraphRuntime;
pub(in crate::domain_computation::primary_graph) mod installation;
mod publication_entry;
mod resource_support;

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
    pub(crate) inner: WorthQueryPrimaryGraphApplicationRuntimeInner<Schema>,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(crate) fn inner(&self) -> &WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
        &self.inner
    }

    pub(crate) fn inner_mut(
        &mut self,
    ) -> &mut WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
        &mut self.inner
    }

    pub(in crate::domain_computation) fn operation_requires_application_program<
        Operation: 'static,
    >(
        &self,
    ) -> bool {
        self.inner
            .operation_requires_application_program::<Operation>()
    }

    pub(in crate::domain_computation) fn operation_requires_workflow_authority<
        Operation: 'static,
    >(
        &self,
    ) -> bool {
        self.inner
            .operation_requires_workflow_authority::<Operation>()
    }

    pub fn requires_application_program<Binding>(&self) -> bool
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        self.inner.requires_application_program::<Binding>()
    }

    pub(in crate::domain_computation) fn is_branch_adoption_original(
        &self,
        effects: &worth_runtime_world::facade::WorldRecoveredUnpublishedEffects,
    ) -> Result<bool, super::query_obligations::QueryObligationDenial> {
        self.inner.is_branch_adoption_original(effects)
    }

    pub(super) fn installed_program_support(
        &self,
    ) -> Option<&super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        self.inner.installed_program_support()
    }

    #[inline]
    pub(in crate::domain_computation) fn authentication_is_expired(
        &self,
        valid_until: std::time::Instant,
    ) -> bool {
        self.inner.authentication_is_expired(valid_until)
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
        self.inner.producer::<Binding>()
    }

    pub fn conditional<Binding>(&self) -> Option<Arc<Binding::Installed>>
    where
        Binding: super::application_contribution::WorthQueryApplicationConditionalBinding<Schema>,
    {
        use crate::equality_type_id::EqualityTypeId as TypeId;
        self.inner
            .installed_conditionals
            .entry_value(
                Binding::IDENTITY,
                TypeId::of::<Binding>(),
                TypeId::of::<Binding::Installed>(),
            )
            .and_then(|value| value.downcast::<Binding::Installed>().ok())
    }

    pub fn installed_schema(&self) -> &WorthQueryInstalledApplicationSchema<Schema> {
        self.inner.installed_schema()
    }

    #[inline]
    pub fn publication(&self) -> &WorthQueryPrimaryGraphPublication {
        self.inner.publication()
    }

    #[inline]
    pub fn checkpoint_restore_work(
        &self,
    ) -> Option<worth_relational::facade::durability::CheckpointRestoreWork> {
        self.inner.checkpoint_restore_work()
    }

    #[inline]
    pub fn granular_invalidation_installation(
        &self,
    ) -> super::WorthQueryGranularInvalidationInstallation {
        self.inner.granular_invalidation_installation()
    }

    #[inline]
    pub fn application_query_basis_observer(
        &self,
    ) -> super::application_query::WorthQueryApplicationBasisObserver {
        self.inner.application_query_basis_observer()
    }

    #[inline]
    pub fn capability_plan_compilation_evidence(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryCapabilityPlanCompilationEvidence {
        self.inner.capability_plan_compilation_evidence()
    }

    #[doc(hidden)]
    #[inline]
    pub fn workflow_compilation_reuse_counters(
        &self,
    ) -> super::WorthQueryWorkflowCompilationReuseCounters {
        self.inner.workflow_compilation_reuse_counters()
    }

    #[doc(hidden)]
    #[inline]
    pub fn workflow_instance_progress_counters(
        &self,
    ) -> super::WorthQueryWorkflowInstanceProgressCounters {
        self.inner.workflow_instance_progress_counters()
    }

    #[inline]
    pub fn close_live_delivery(&self) -> WorthQueryApplicationLiveDeliveryCloseReceipt {
        self.inner.close_live_delivery()
    }
}

mod court_doors_for_test;
#[cfg(test)]
#[path = "application_runtime_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

#[cfg(test)]
fn collect_owned_inline_type_sizes(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryPrimaryGraphApplicationRuntime<()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_runtime.rs", "WorthQueryPrimaryGraphApplicationRuntime", 128 * 1024);
        owner.field::<conditional_cleanup::ConditionalRuntimeCleanup<()>>("conditional_cleanup");
        owner.field::<WorthQueryPrimaryGraphApplicationRuntimeInner<()>>("inner");
    }
}
