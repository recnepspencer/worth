use super::*;
#[cfg(test)]
use crate::domain_capabilities::WorthQueryInvariantCatalogRegistrationArtifact;
use worth_relational::facade::runtime::{
    CustomInvariantRegistration, InvariantCatalog, RelationalRuntimeBuilder,
};

mod conditional_execution;
mod construction;
mod consumer_support;
mod declaration_authority;
pub use declaration_authority::{
    WorthQueryDeclarationAuthorityRuntime, WorthQueryDeclarationAuthorityRuntimeBuilder,
};
mod domain_operation_executors;
mod domain_packages;
mod graph_participation;
mod host_installation;
mod lowering;
mod owned_async_source;
mod primary_graph;
mod product_bridge;
mod product_source;
mod workflow_parallel_admission;
mod workflow_stage_executors;
pub use primary_graph::{
    WorthQueryPrimaryGraphConfiguration, WorthQueryPrimaryGraphConfigurationDenial,
    WorthQueryPrimaryGraphConfigurationDenialKind,
};

#[derive(Default)]
struct QueuedInvariantRegistrations {
    invariant_catalog: Option<InvariantCatalog>,
    custom_invariants: Vec<CustomInvariantRegistration>,
}

impl QueuedInvariantRegistrations {
    fn is_empty(&self) -> bool {
        self.invariant_catalog.is_none() && self.custom_invariants.is_empty()
    }

    #[cfg(test)]
    fn push_invariant_catalog(&mut self, invariant_catalog: InvariantCatalog) {
        match &mut self.invariant_catalog {
            Some(existing) => {
                existing
                    .registrations
                    .extend(invariant_catalog.registrations);
                *existing = existing.canonicalized();
            }
            None => {
                self.invariant_catalog = Some(invariant_catalog.canonicalized());
            }
        }
    }

    fn lower_into_relational_runtime(self) -> RelationalRuntime {
        let mut builder = RelationalRuntimeBuilder::new();
        if let Some(invariant_catalog) = self.invariant_catalog {
            builder = builder.invariant_catalog(invariant_catalog.canonicalized());
        }
        for custom_invariant in self.custom_invariants {
            builder = builder.custom_invariant(custom_invariant);
        }
        builder.build()
    }
}

pub struct WorthQueryRuntimeBuilder {
    product_world_resources:
        worth_query_execution::facade::integration::WorthQueryProductWorldResources,
    backend: Option<Result<Box<dyn WorthQueryRuntimeBackend>, WorthQueryRuntimeError>>,
    backend_parts: WorthQueryRuntimeBackendParts,
    queued_invariant_registrations: QueuedInvariantRegistrations,
    pending_domain_installations: crate::domain_installation::WorthQueryPendingDomainInstallations,
    pending_graph_participations: crate::domain_installation::WorthQueryPendingGraphParticipations,
    pending_domain_operation_executors:
        crate::domain_installation::WorthQueryPendingDomainOperationExecutors,
    pending_workflow_stage_executors:
        crate::domain_installation::WorthQueryPendingWorkflowStageExecutors,
    pending_workflow_parallel_admission_providers:
        crate::domain_installation::WorthQueryPendingWorkflowParallelAdmissionProviders,
    consumer_support_postures:
        [Option<crate::domain_installation::WorthQueryConsumerSupportPosture>;
            crate::domain_installation::WorthQueryConsumerSupportDimension::COUNT],
    native_aspect_contracts:
        crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry,
    conditional_runtime_bridge: Option<worth_runtime_bridge::facade::RuntimeBridge>,
    conditional_signal_graph: Option<Box<worth_signal::facade::SignalGraph>>,
    conditional_execution_resources: Option<super::WorthQueryConditionalExecutionResources>,
    pending_relational_product_bridge:
        Option<product_bridge::WorthQueryPendingRelationalProductBridge>,
    pending_conditional_installations:
        Vec<Box<dyn crate::domain_installation::PendingConditionalInstallation>>,
    pending_owned_async_declarations: Vec<super::WorthQueryOwnedAsyncRequestDeclaration>,
    pending_primary_graph_installation:
        Option<Box<dyn primary_graph::PendingPrimaryGraphInstallation>>,
    primary_runtime_invalidation_installation: Option<
        worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    >,
    host_execution_installation:
        Option<worth_query_execution::facade::runtime::WorthQueryExecutionRuntimeInstallation>,
}

pub use host_installation::{
    WorthQueryHostRuntimeCompletionError, WorthQueryHostRuntimeInstallationCompletion,
    WorthQueryHostRuntimeInstallationDenial, WorthQueryHostRuntimeInstallationDenialKind,
    WorthQueryHostRuntimeInstallationPlan, WorthQueryHostRuntimeInstallationRequest,
};

impl WorthQueryRuntimeBuilder {
    pub fn new(
        product_world_resources: worth_query_execution::facade::integration::WorthQueryProductWorldResources,
    ) -> Self {
        Self {
            product_world_resources,
            backend: None,
            backend_parts: WorthQueryRuntimeBackendParts::default(),
            queued_invariant_registrations: QueuedInvariantRegistrations::default(),
            pending_domain_installations: Default::default(),
            pending_graph_participations: Default::default(),
            pending_domain_operation_executors: Default::default(),
            pending_workflow_stage_executors: Default::default(),
            pending_workflow_parallel_admission_providers: Default::default(),
            consumer_support_postures: Default::default(),
            native_aspect_contracts: Default::default(),
            conditional_runtime_bridge: None,
            conditional_signal_graph: None,
            conditional_execution_resources: None,
            pending_relational_product_bridge: None,
            pending_conditional_installations: Vec::new(),
            pending_owned_async_declarations: Vec::new(),
            pending_primary_graph_installation: None,
            primary_runtime_invalidation_installation: None,
            host_execution_installation: None,
        }
    }

    /// Shares the execution-owned primary graph that produces granular
    /// invalidation batches with this Query runtime backend.
    pub fn primary_runtime_granular_invalidations(
        mut self,
        installation: worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    ) -> Self {
        self.primary_runtime_invalidation_installation = Some(installation);
        self
    }

    pub fn backend(mut self, backend: impl WorthQueryRuntimeBackend + 'static) -> Self {
        self.backend = Some(Ok(Box::new(backend)));
        self
    }

    pub fn relational_runtime(mut self, runtime: RelationalRuntime) -> Self {
        self.backend_parts = self.backend_parts.relational_runtime(runtime);
        self
    }

    pub fn relational_source_owner(
        mut self,
        owner: worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
    ) -> Self {
        self.backend_parts = self.backend_parts.relational_source_owner(owner);
        self
    }

    #[cfg(test)]
    pub(crate) fn invariant_catalog(mut self, invariant_catalog: InvariantCatalog) -> Self {
        self.queued_invariant_registrations
            .push_invariant_catalog(invariant_catalog);
        self
    }

    #[cfg(test)]
    pub(crate) fn invariant_registration_artifact(
        mut self,
        artifact: WorthQueryInvariantCatalogRegistrationArtifact,
    ) -> Self {
        self.queued_invariant_registrations
            .push_invariant_catalog(artifact.invariant_catalog().clone());
        self
    }

    #[cfg(test)]
    pub(crate) fn register_invariant<R>(mut self, rule: R) -> Result<Self, WorthQueryRuntimeError>
    where
        R: CustomInvariantRule + std::panic::UnwindSafe + 'static,
    {
        let registration = CustomInvariantRegistration::new(rule).map_err(|error| {
            WorthQueryRuntimeError::InvariantRegistration {
                stage: "query_builder_custom_invariant_registration",
                message: format!("{error:?}"),
            }
        })?;
        self.queued_invariant_registrations
            .custom_invariants
            .push(registration);
        Ok(self)
    }

    pub fn runtime_bridge(mut self, bridge: RuntimeBridge) -> Self {
        self.conditional_runtime_bridge = Some(bridge.clone());
        self.backend_parts = self.backend_parts.runtime_bridge(bridge);
        self
    }

    pub fn schema_adapter(
        mut self,
        adapter: impl WorthQueryRuntimeSchemaAdapter + 'static,
    ) -> Self {
        self.backend_parts = self.backend_parts.schema_adapter(adapter);
        self
    }

    pub fn source_adapter(
        mut self,
        adapter: impl WorthQueryRuntimeSourceAdapter + 'static,
    ) -> Self {
        self.backend_parts = self.backend_parts.source_adapter(adapter);
        self
    }

    pub fn snapshot_identity(
        mut self,
        adapter: impl WorthQueryRuntimeSnapshotIdentityAdapter + 'static,
    ) -> Self {
        self.backend_parts = self.backend_parts.snapshot_identity(adapter);
        self
    }

    pub fn existing_truth_verification(
        mut self,
        adapter: impl WorthQueryRuntimeExistingTruthVerificationAdapter + 'static,
    ) -> Self {
        self.backend_parts = self.backend_parts.existing_truth_verification(adapter);
        self
    }

    pub fn write_authority(
        mut self,
        authority: impl WorthQueryRuntimeWriteAuthorityAdapter + 'static,
    ) -> Self {
        self.backend_parts = self.backend_parts.write_authority(authority);
        self
    }

    pub fn signal_sink(mut self, sink: impl WorthQueryRuntimeSignalSinkAdapter + 'static) -> Self {
        self.backend_parts = self.backend_parts.signal_sink(sink);
        self
    }

    pub fn subscription_activation(
        mut self,
        adapter: impl WorthQueryRuntimeSubscriptionActivationAdapter + 'static,
    ) -> Self {
        self.backend_parts = self.backend_parts.subscription_activation(adapter);
        self
    }

    pub fn preview_basis(
        mut self,
        adapter: impl WorthQueryRuntimePreviewBasisAdapter + 'static,
    ) -> Self {
        self.backend_parts = self.backend_parts.preview_basis(adapter);
        self
    }

    pub fn inspector_evidence(
        mut self,
        adapter: impl WorthQueryRuntimeInspectorEvidenceAdapter + 'static,
    ) -> Self {
        self.backend_parts = self.backend_parts.inspector_evidence(adapter);
        self
    }

    pub fn declaration_initialization(
        mut self,
        adapter: impl WorthQueryRuntimeDeclarationInitializationAdapter + 'static,
    ) -> Self {
        self.backend_parts = self.backend_parts.declaration_initialization(adapter);
        self
    }

    pub fn intent_authority(
        mut self,
        adapter: impl WorthQueryIntentAuthorityAdapter + 'static,
    ) -> Self {
        self.backend_parts = self.backend_parts.intent_authority(adapter);
        self
    }

    pub fn support_profile(mut self, profile: WorthQueryRuntimeSupportProfile) -> Self {
        self.backend_parts = self.backend_parts.support_profile(profile);
        self
    }

    pub fn aspect_contract(
        mut self,
        contract: worth_foundational::facade::AspectContract,
    ) -> Result<Self, crate::runtime::WorthQueryAspectContractRegistrationDenial> {
        self.native_aspect_contracts.install(contract)?;
        Ok(self)
    }

    pub fn aspect_contracts(
        mut self,
        contracts: impl IntoIterator<Item = worth_foundational::facade::AspectContract>,
    ) -> Result<Self, crate::runtime::WorthQueryAspectContractRegistrationDenial> {
        for contract in contracts {
            self.native_aspect_contracts.install(contract)?;
        }
        Ok(self)
    }

    pub fn build_backend_from_parts(mut self) -> Self {
        self.queue_installed_domain_substrates();
        if self.backend.is_some() {
            self.backend = Some(Err(WorthQueryRuntimeError::InvariantRegistration {
                stage: "runtime_backend_authority_selection",
                message: "build_backend_from_parts() cannot replace an explicit runtime backend selected through backend(...); choose one backend authority path".to_string(),
            }));
            self.backend_parts = WorthQueryRuntimeBackendParts::new();
            self.queued_invariant_registrations = QueuedInvariantRegistrations::default();
            return self;
        }
        if let Err(error) = self.lower_queued_invariant_registrations_into_backend_parts() {
            self.backend = Some(Err(error));
            self.backend_parts = WorthQueryRuntimeBackendParts::new();
            self.queued_invariant_registrations = QueuedInvariantRegistrations::default();
            return self;
        }
        if let Err(error) = self.install_pending_relational_product_bridge() {
            self.backend = Some(Err(error));
            self.backend_parts = WorthQueryRuntimeBackendParts::new();
            self.queued_invariant_registrations = QueuedInvariantRegistrations::default();
            return self;
        }
        self.backend = Some(self.lower_bridge_backed_backend_from_parts());
        self.backend_parts = WorthQueryRuntimeBackendParts::new();
        self.queued_invariant_registrations = QueuedInvariantRegistrations::default();
        self
    }
}
