use std::sync::{Arc, Mutex};

use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{
    RelationalBridgeBranchHeadLease, RelationalBridgeCanonicalSubscription,
    RelationalBridgeSourceConfigurationError, RuntimeBridgeRelationalSource,
};

#[derive(Debug)]
pub enum WorthQueryRelationalSourceInstallationDenial {
    BridgeConfiguration(RelationalBridgeSourceConfigurationError),
    CompanionRegistration(worth_relational::facade::mvcc::PublicationCompanionRegistrationStop),
}

/// Shared custody of one installed Relational owner and its exact Bridge registry.
///
/// This integration surface accepts the backend's existing runtime. It neither
/// creates an application schema nor derives source authority from an identity string.
#[doc(hidden)]
#[derive(Clone)]
pub struct WorthQueryRelationalSourceOwner {
    pub(super) runtime: Arc<Mutex<RelationalRuntime>>,
    pub(super) source: RuntimeBridgeRelationalSource,
    pub(super) bridge_head: Arc<Mutex<Option<RelationalBridgeBranchHeadLease>>>,
    pub(in crate::domain_computation) invalidation_owner:
        Arc<crate::domain_computation::primary_graph::SourceInvalidationOwner>,
    _canonical_subscription: RelationalBridgeCanonicalSubscription,
}

impl WorthQueryRelationalSourceOwner {
    pub fn new(
        runtime: RelationalRuntime,
        graph_role: impl Into<Arc<str>>,
        invalidation_resources: super::super::WorthQueryInvalidationResources,
    ) -> Result<Self, WorthQueryRelationalSourceInstallationDenial> {
        let runtime_instance_id = runtime.main_branch_identity().runtime_instance_id();
        let runtime = Arc::new(Mutex::new(runtime));
        let source =
            RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), graph_role)
                .map_err(WorthQueryRelationalSourceInstallationDenial::BridgeConfiguration)?;
        let invalidation_owner = Arc::new(
            crate::domain_computation::primary_graph::SourceInvalidationOwner::new(
                invalidation_resources.clone(),
                runtime_instance_id,
            ),
        );
        let pending = source
            .begin_canonical_envelope_subscription()
            .map_err(WorthQueryRelationalSourceInstallationDenial::CompanionRegistration)?;
        let subscription = pending
            .activate(
                invalidation_owner.clone(),
                invalidation_resources.preflight_budget(),
            )
            .map_err(WorthQueryRelationalSourceInstallationDenial::CompanionRegistration)?;
        Ok(Self {
            runtime,
            source,
            bridge_head: Arc::new(Mutex::new(None)),
            invalidation_owner,
            _canonical_subscription: subscription,
        })
    }

    pub fn with_runtime<T>(&self, read: impl FnOnce(&RelationalRuntime) -> T) -> T {
        let runtime = self
            .runtime
            .lock()
            .expect("Relational source owner is available");
        read(&runtime)
    }

    pub fn with_runtime_mut<T>(&self, mutate: impl FnOnce(&mut RelationalRuntime) -> T) -> T {
        let mut runtime = self
            .runtime
            .lock()
            .expect("Relational source owner is available");
        mutate(&mut runtime)
    }

    pub(crate) fn with_runtime_mut_unwind_isolated<T>(
        &self,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> T {
        let mut runtime = self
            .runtime
            .lock()
            .expect("Relational source owner is available");
        let outcome =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| mutate(&mut runtime)));
        drop(runtime);
        match outcome {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    pub fn bridge_source(&self) -> RuntimeBridgeRelationalSource {
        self.source.clone()
    }

    pub(crate) fn bind_truth_partition(
        &mut self,
        graph_role: impl Into<Arc<str>>,
        partition: worth_relational::facade::identity::PartitionId,
        role: worth_foundational::facade::TruthPartitionRole,
    ) -> Result<(), RelationalBridgeSourceConfigurationError> {
        self.source = RuntimeBridgeRelationalSource::for_shared_graph_partition(
            Arc::clone(&self.runtime),
            graph_role,
            partition,
            role,
        )?;
        Ok(())
    }
}
