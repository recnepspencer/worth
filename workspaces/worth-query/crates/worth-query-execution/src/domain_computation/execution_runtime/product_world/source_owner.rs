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
    CompanionPreparation(worth_relational::facade::mvcc::CompanionPreflightStop),
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
    canonical_subscription: RelationalBridgeCanonicalSubscription,
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
        invalidation_owner
            .admit_publication_fallback()
            .map_err(WorthQueryRelationalSourceInstallationDenial::CompanionPreparation)?;
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
            canonical_subscription: subscription,
        })
    }

    /// Resolve only a branch identity here; Native alone selects the current
    /// head while excluding publication through the prepaid lookup install.
    pub(in crate::domain_computation) fn mint_mark_cell_at_head(
        &self,
        branch: &worth_relational::facade::history::BranchId,
        admission: &mut crate::domain_computation::primary_graph::InvalidationEditAdmission,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::output_lineage::HeadCellRegistrationStop,
    > {
        self.with_runtime(|runtime| {
            let identity = runtime.branch_identity(branch).map_err(|_|
                crate::domain_computation::primary_graph::output_lineage::HeadCellRegistrationStop::Native(
                    worth_relational::facade::mvcc::PublicationCompanionRegistrationStop::HeadUnavailable))?;
            self.invalidation_owner.mint_cell_at_head(&self.canonical_subscription, runtime, &identity, admission)
        })
    }

    /// Returns exact snapshot custody to its issuing owner. This is release,
    /// not a source read: even a refused advancement must settle retained facts.
    pub(crate) fn release_query_snapshot(
        &self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        let mut runtime = self
            .runtime
            .lock()
            .expect("Relational source owner is available");
        crate::relational_snapshot_release::release_query_snapshot(&mut runtime, snapshot);
    }

    pub fn with_runtime<T>(&self, read: impl FnOnce(&RelationalRuntime) -> T) -> T {
        let runtime = self
            .runtime
            .lock()
            .expect("Relational source owner is available");
        #[cfg(feature = "test-query-execution-observer")]
        super::read_observation::record_read();
        read(&runtime)
    }

    pub fn with_runtime_mut<T>(&self, mutate: impl FnOnce(&mut RelationalRuntime) -> T) -> T {
        let mut runtime = self
            .runtime
            .lock()
            .expect("Relational source owner is available");
        #[cfg(feature = "test-query-execution-observer")]
        super::read_observation::record_read();
        mutate(&mut runtime)
    }

    /// The prepared read needs immutable access, while the installed owner's
    /// mutex remains exclusive. Freeing this lock requires a native pinned
    /// read capability; cloning a runtime would create a second owner.
    pub(crate) fn with_runtime_unwind_isolated<T>(
        &self,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T {
        self.with_runtime_mut_unwind_isolated(|runtime| read(runtime))
    }

    pub(crate) fn with_runtime_mut_unwind_isolated<T>(
        &self,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> T {
        let mut runtime = self
            .runtime
            .lock()
            .expect("Relational source owner is available");
        #[cfg(feature = "test-query-execution-observer")]
        super::read_observation::record_read();
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
