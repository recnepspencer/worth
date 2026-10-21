use std::sync::Arc;
use std::sync::Mutex;

use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::RelationalBridgeBranchHeadLease;
use worth_runtime_bridge::facade::RelationalBridgeCanonicalSubscription;
use worth_runtime_bridge::facade::RelationalBridgeSourceConfigurationError;
use worth_runtime_bridge::facade::RuntimeBridgeRelationalSource;

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
    runtime: Arc<Mutex<RelationalRuntime>>,
    pub(in crate::domain_computation) runtime_instance_id: u64,
    pub(super) source: RuntimeBridgeRelationalSource,
    pub(super) bridge_head: Arc<Mutex<Option<RelationalBridgeBranchHeadLease>>>,
    pub(in crate::domain_computation) invalidation_owner:
        Arc<crate::domain_computation::primary_graph::SourceInvalidationOwner>,
    canonical_subscription: RelationalBridgeCanonicalSubscription,
    #[cfg(test)]
    read_acquisitions: Arc<std::sync::atomic::AtomicU64>,
    #[cfg(test)]
    _bridge_lock_observations: Arc<
        Mutex<
            Vec<worth_relational::facade::change_source::RelationalRuntimeLockObservationForTest>,
        >,
    >,
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
            .activate(invalidation_owner.clone())
            .map_err(WorthQueryRelationalSourceInstallationDenial::CompanionRegistration)?;
        #[cfg(test)]
        let read_acquisitions = Arc::new(std::sync::atomic::AtomicU64::new(0));
        #[cfg(test)]
        let bridge_lock_observations = {
            let count = Arc::clone(&read_acquisitions);
            Arc::new(Mutex::new(vec![source.observe_runtime_locks_for_test(
                move || {
                    count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                },
            )]))
        };
        Ok(Self {
            runtime,
            runtime_instance_id,
            source,
            bridge_head: Arc::new(Mutex::new(None)),
            invalidation_owner,
            canonical_subscription: subscription,
            #[cfg(test)]
            read_acquisitions,
            #[cfg(test)]
            _bridge_lock_observations: bridge_lock_observations,
        })
    }

    fn lock_runtime(&self) -> std::sync::MutexGuard<'_, RelationalRuntime> {
        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        #[cfg(test)]
        self.read_acquisitions
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        runtime
    }
    #[cfg(test)]
    pub(super) fn lock_is_held_for_test(&self) -> bool {
        self.runtime.try_lock().is_err()
    }
    #[cfg(test)]
    pub(super) fn lock_acquisitions_for_test(&self) -> u64 {
        self.read_acquisitions
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Construction reads only the owner's stable branch identity.
    ///
    /// Resolve only a branch identity here; Native alone selects the current
    /// head while excluding publication through the prepaid lookup install.
    pub(in crate::domain_computation) fn mint_mark_cell_at_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &worth_relational::facade::history::BranchId,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::output_lineage::HeadCellRegistrationStop,
    > {
        self.with_runtime(execution, |runtime| {
            let identity = runtime.branch_identity(branch).map_err(|_|
                crate::domain_computation::primary_graph::output_lineage::HeadCellRegistrationStop::Native(
                    worth_relational::facade::mvcc::PublicationCompanionRegistrationStop::HeadUnavailable))?;
            self.invalidation_owner.mint_cell_at_head(&self.canonical_subscription, runtime, &identity)
        })
    }

    /// Returns exact snapshot custody to its issuing owner. This is release,
    /// not a source read: even a refused advancement must settle retained facts.
    /// Mutex poison cannot prevent physical custody release after caller unwind.
    #[doc(hidden)]
    pub fn release_query_snapshot(
        &self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        let mut runtime = self.lock_runtime();
        crate::relational_snapshot_release::release_query_snapshot(&mut runtime, snapshot);
    }

    pub(in crate::domain_computation) fn release_component_basis(
        &self,
        retention: worth_relational::facade::branch::RelationalBranchRetentionLease,
    ) -> Result<
        worth_relational::facade::branch::RelationalBranchRetentionReleaseReceipt,
        worth_relational::facade::branch::RelationalBranchRetentionReleaseDenial,
    > {
        let runtime = self.lock_runtime();
        runtime.release_component_basis(retention)
    }

    /// Metadata read: fixed-size counters, without row traversal or request work.
    pub(in crate::domain_computation) fn custom_invariant_candidate_input_counters(
        &self,
    ) -> worth_relational::facade::runtime::RelationalCandidateInputCounters {
        self.lock_runtime()
            .custom_invariant_candidate_input_counters()
    }

    pub fn with_runtime<T>(
        &self,
        // Required caller request proves custody for this callback.
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T {
        let runtime = self.lock_runtime();
        super::read_observation::record_read_for_test();
        read(&runtime)
    }

    pub fn with_runtime_mut<T>(
        &self,
        // Required caller request proves custody for this callback.
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> T {
        let mut runtime = self.lock_runtime();
        super::read_observation::record_read_for_test();
        mutate(&mut runtime)
    }

    pub(crate) fn with_runtime_mut_unwind_isolated<T>(
        &self,
        // Required caller request proves custody for this callback.
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> T {
        let mut runtime = self.lock_runtime();
        super::read_observation::record_read_for_test();
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
        self.source = self
            .source
            .clone()
            .with_graph_partition(graph_role, partition, role)?;
        Ok(())
    }
}
