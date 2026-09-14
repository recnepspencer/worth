use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport;
use crate::domain_computation::authorization::WorthQueryRuntimeClock;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};
use crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry;
use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource;

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
#[cfg(feature = "test-world-operation-control")]
pub(in crate::domain_computation::primary_graph) use operation_control::WorthQueryApplicationAttemptOperationControl;

pub(in crate::domain_computation) use external_dispatch_attempt::WorthQueryExternalDispatchAttemptOrdinal;

/// Purpose-scoped application runtime published from one typed primary graph.
///
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
    pub(super) installed_conditionals:
        super::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<Schema>,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
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

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn publish_application_runtime(
        self,
        runtime: WorthQueryExecutionRuntime,
        authority: WorthQueryExecutionInstallationAuthority,
        installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
        conditional_evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        installation::require_no_conditional_bindings(&runtime, &installed_schema)?;
        installation::publish_application_runtime_with_clock(
            installation::ApplicationRuntimePublication {
                bootstrap: self,
                runtime,
                authority,
                installed_schema,
                authorization_clock: WorthQueryRuntimeClock::system(),
                fault_port: super::provider::fault_port::production_fault_port(),
                conditional_evaluation_budget,
            },
        )
    }

    /// Begins the sole primary-graph conditional publication progression.
    ///
    /// Complete provider, clock, and reconstruction bindings are accumulated
    /// here before the application runtime can become visible.
    pub fn conditional_application_runtime_installation(
        self,
        runtime: WorthQueryExecutionRuntime,
        authority: WorthQueryExecutionInstallationAuthority,
        installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
        conditional_evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    ) -> Result<
        super::conditional_operation::WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
        super::conditional_operation::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        super::conditional_operation::WorthQueryConditionalApplicationRuntimeInstallation::new(
            installation::ApplicationRuntimePublication {
                bootstrap: self,
                runtime,
                authority,
                installed_schema,
                authorization_clock: WorthQueryRuntimeClock::system(),
                fault_port: super::provider::fault_port::production_fault_port(),
                conditional_evaluation_budget,
            },
        )
    }

    /// Publishes one application runtime with a host-installed trusted-time
    /// mechanism.
    ///
    /// The source is fixed for the lifetime of the returned runtime. It grants
    /// no Query authority and is never exposed to operation callers.
    pub fn publish_application_runtime_with_authorization_time_source(
        self,
        runtime: WorthQueryExecutionRuntime,
        authority: WorthQueryExecutionInstallationAuthority,
        installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
        conditional_evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
        source: impl WorthQueryRuntimeTimeSource,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        self.publish_application_runtime_with_ports(
            runtime,
            authority,
            installed_schema,
            conditional_evaluation_budget,
            source,
            super::provider::fault_port::production_fault_port(),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn publish_application_runtime_with_ports(
        self,
        runtime: WorthQueryExecutionRuntime,
        authority: WorthQueryExecutionInstallationAuthority,
        installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
        conditional_evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
        source: impl WorthQueryRuntimeTimeSource,
        fault_port: Arc<dyn super::provider::fault_port::WorthQueryPrimaryGraphFaultPort>,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        installation::publish_application_runtime_with_clock(
            installation::ApplicationRuntimePublication {
                bootstrap: self,
                runtime,
                authority,
                installed_schema,
                authorization_clock: WorthQueryRuntimeClock::from_source(source),
                fault_port,
                conditional_evaluation_budget,
            },
        )
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

    pub(in crate::domain_computation::primary_graph) fn release_conditional_runtime_resources(
        &mut self,
    ) {
        self.bridge
            .conditional_lifecycle()
            .close_conditional_resources();
        *self
            .conditional_operations
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Default::default();
        self.primary_provider.replace_conditional_commit_routes(
            std::iter::empty(),
            false,
            std::iter::empty(),
        );
    }
}

impl<Schema> Drop for WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn drop(&mut self) {
        self.release_conditional_runtime_resources();
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
    pub fn close_live_delivery(&self) {
        self.primary_provider.live_delivery.close();
    }
}
