use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport;
use crate::domain_computation::authorization::WorthQueryRuntimeClock;
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntime;
use crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry;

use super::provider::WorthQueryPrimaryGraphProvider;
use super::{
    authentication_clock::WorthQueryAuthenticationClock, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
    WorthQueryPrimaryGraphPublication,
};
use crate::domain_computation::authorization::WorthQueryInstalledAuthorizationRegistry;

#[cfg(any(
    test,
    feature = "test-primary-graph-faults",
    feature = "test-durability-faults"
))]
mod certification_controls;
mod certification_cost;
mod conditional_cleanup;
mod external_dispatch_attempt;
pub use certification_cost::{
    WorthQueryCertificationApplicationWork, WorthQueryCertificationCostObservation,
    WorthQueryCertificationCostRuntimeExt, WorthQueryCertificationCostScope,
    WorthQueryCertificationWorldHistory, WorthQueryCertificationWorldRetention,
};
mod graph_participation;
pub(in crate::domain_computation::primary_graph) mod installation;
#[cfg(feature = "test-world-operation-control")]
mod operation_control;
mod publication_entry;
#[cfg(feature = "test-world-operation-control")]
pub(in crate::domain_computation::primary_graph) use operation_control::WorthQueryApplicationAttemptOperationControl;

pub(in crate::domain_computation) use external_dispatch_attempt::WorthQueryExternalDispatchAttemptOrdinal;

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
    pub(in crate::domain_computation) runtime: WorthQueryExecutionRuntime,
    pub(in crate::domain_computation) installed_schema:
        WorthQueryInstalledApplicationSchema<Schema>,
    pub(super) application_readiness_schema_token: String,
    publication: WorthQueryPrimaryGraphPublication,
    checkpoint_restore_work: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
    pub(in crate::domain_computation) authorization: WorthQueryInstalledAuthorizationRegistry,
    pub(in crate::domain_computation) authorization_clock: Arc<WorthQueryRuntimeClock>,
    authentication_clock: WorthQueryAuthenticationClock,
    pub(super) relational_branch_identity:
        worth_relational::facade::branch::RelationalBranchIdentity,
    pub(crate) bridge: super::managed_bridge::WorthQueryInstalledApplicationBridge,
    pub(crate) product_runtime:
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    pub(super) basis_leases:
        super::application_query::resource_lifecycle::WorthQueryApplicationBasisRegistry,
    pub(super) granular_invalidation: super::WorthQueryGranularInvalidationInstallation,
    pub(super) conditional_operations: std::sync::Mutex<
        super::conditional_operation::WorthQueryConditionalOperationRegistry<Schema>,
    >,
    pub(crate) primary_provider: std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    pub(super) primary_graph_authority:
        worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    pub(super) result_buffers:
        super::application_query::resource_lifecycle::WorthQueryApplicationResultBufferRegistry,
    pub(super) source_meanings:
        super::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry,
    pub(super) next_external_dispatch_attempt: AtomicU64,
    pub(super) external_effect_transport:
        std::sync::OnceLock<std::sync::Arc<dyn WorthQueryExternalEffectTransport>>,
    /// Instance-local recovery-handle live set (Q8.9 / R8.29).
    pub(crate) recovery_handles: Arc<WorthQueryRecoveryHandleRegistry>,
    pub(super) mutation_handlers: super::handler::InstalledMutationHandlerRegistry<Schema>,
    pub(super) mutation_projection:
        super::WorthQueryApplicationInvariantProjectionAuthority<Schema>,
    pub(super) installed_producers:
        super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>,
    pub(super) output_producer_routes:
        super::application_contribution::WorthQueryInstalledOutputProducerRoutes,
    pub(super) output_readiness_routes:
        super::application_contribution::WorthQueryInstalledOutputReadinessRoutes,
    pub(super) next_output_producer_attempt: AtomicU64,
    pub(super) next_application_mutation_partition: AtomicU32,
    pub(super) output_demands: super::application_output_demand::WorthQueryOutputDemandRegistry,
    pub(super) recovered_outputs: super::application_output_demand::WorthQueryRecoveredOutputs,
    pub(super) program_required_bindings: std::collections::BTreeSet<std::any::TypeId>,
    pub(super) program_required_operations: std::collections::BTreeSet<std::any::TypeId>,
    pub(super) program_support:
        Option<super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>>,
    pub(super) installed_conditionals:
        super::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<Schema>,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
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
                .contains(&std::any::TypeId::of::<Binding>())
            || self
                .program_required_operations
                .contains(&std::any::TypeId::of::<Binding::Operation>())
    }

    /// The immutable program support this host admitted at installation, when
    /// it admitted any.
    pub(super) fn installed_program_support(
        &self,
    ) -> Option<&super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        self.program_support.as_ref()
    }

    pub(super) fn issue_application_mutation_partition(
        &self,
    ) -> Option<worth_relational::facade::identity::PartitionId> {
        self.next_application_mutation_partition
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .ok()
            .map(worth_relational::facade::identity::PartitionId)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn authentication_is_expired(
        &self,
        valid_until: std::time::Instant,
    ) -> bool {
        self.authentication_clock.is_expired(valid_until)
    }

    #[cfg(test)]
    pub(crate) fn fix_authentication_time(&mut self, now: std::time::Instant) {
        self.authentication_clock = WorthQueryAuthenticationClock::fixed(now);
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

    pub fn publication(&self) -> &WorthQueryPrimaryGraphPublication {
        &self.publication
    }

    /// Native checkpoint readmission work for this installation, if it was
    /// restored. This is diagnostic only and grants no recovered authority.
    pub fn checkpoint_restore_work(
        &self,
    ) -> Option<worth_relational::facade::durability::CheckpointRestoreWork> {
        self.checkpoint_restore_work
    }

    /// Retains the opaque installation needed to bind Query maintenance to
    /// this exact primary application runtime.
    pub fn granular_invalidation_installation(
        &self,
    ) -> super::WorthQueryGranularInvalidationInstallation {
        self.granular_invalidation.current()
    }

    pub fn result_buffer_observer(
        &self,
    ) -> super::application_query::WorthQueryApplicationResultBufferObserver {
        self.result_buffers.observer()
    }

    pub fn application_query_basis_observer(
        &self,
    ) -> super::application_query::WorthQueryApplicationBasisObserver {
        self.basis_leases.observer()
    }

    pub fn capability_plan_compilation_evidence(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryCapabilityPlanCompilationEvidence {
        self.authorization.capability_compilation()
    }

    #[doc(hidden)]
    pub fn workflow_compilation_reuse_counters(
        &self,
    ) -> super::WorthQueryWorkflowCompilationReuseCounters {
        self.runtime
            .retain_primary_graph_integration_handle()
            .expect("a published application runtime retains its primary graph")
            .workflow_compilation_reuse_counters()
    }

    #[doc(hidden)]
    pub fn workflow_instance_progress_counters(
        &self,
    ) -> super::WorthQueryWorkflowInstanceProgressCounters {
        self.runtime
            .retain_primary_graph_integration_handle()
            .expect("a published application runtime retains its primary graph")
            .workflow_instance_progress_counters()
    }

    /// Read-only physical candidate-input observation for product integration
    /// tests; it does not expose a Relational runtime or graph read authority.
    #[doc(hidden)]
    pub fn custom_invariant_candidate_input_counters(
        &self,
    ) -> worth_relational::facade::runtime::RelationalCandidateInputCounters {
        self.primary_provider
            .graph
            .with_runtime(|runtime| runtime.custom_invariant_candidate_input_counters())
    }

    pub(in crate::domain_computation) fn graph_work_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.primary_provider.application_resource_support()
    }

    pub(in crate::domain_computation) fn graph_work_provider_identity(&self) -> &str {
        self.primary_graph_authority.provider_identity()
    }

    /// Closes ordinary live delivery without closing the authoritative graph.
    /// Later commits retain their compact idempotency causality but no longer
    /// enter this runtime's delivery ring.
    pub fn close_live_delivery(&self) -> WorthQueryApplicationLiveDeliveryCloseReceipt {
        WorthQueryApplicationLiveDeliveryCloseReceipt {
            remaining_live_consumers: self.primary_provider.live_delivery.close(),
        }
    }
}

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
