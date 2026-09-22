use super::*;

mod closeout;
mod continuity_planning;
mod debug;
mod diagnostics;
mod historical_and_replay;
mod merge;
mod policy;
mod routing_and_bulk;
mod source;
mod speculation;
mod stream;
mod structural;
mod subscription;
mod writeback;

#[derive(Clone)]
pub struct RuntimeBridge {
    pub(crate) policy: BridgeRuntimePolicy,
    pub(crate) diagnostics: BridgeDiagnosticsFacade,
    pub(crate) diagnostic_sink: Arc<dyn DiagnosticSink>,
    pub(crate) committed_patch_source: Arc<dyn CommittedPatchSource>,
    pub(crate) authoritative_source_profile:
        Option<crate::input::envelope::BridgeAuthoritativeSourceProfile>,
    pub(crate) snapshot_read_source: Arc<dyn SnapshotReadSource>,
    pub(crate) snapshot_reader_pool: Option<Arc<dyn SnapshotReaderPool>>,
    pub(crate) signal_sink: Arc<dyn InvalidationSink>,
    pub(crate) truth_branch_head_source: Option<Arc<dyn TruthBranchHeadSource>>,
    pub(crate) continuity_lineage_source: Option<Arc<dyn ContinuityLineageSource>>,
    pub(crate) writeback_authority: Option<Arc<dyn TruthWritebackAuthority>>,
    pub(crate) source_registry: AdmittedSourceRegistry,
    pub(crate) source_adapter: Option<Arc<dyn BridgeSourceAdapter>>,
    pub(crate) structural_registry: AdmittedStructuralRegistry,
    pub(crate) merge_registry: AdmittedMergeRegistry,
    pub(crate) mapping_registry: FrozenMappingRegistry,
    pub(crate) aspect_registry: FrozenAspectMappingRegistry,
    pub(crate) subscription_family_registry: FrozenSubscriptionFamilyRegistry,
    pub(crate) signal_runtime_key: u64,
    pub(crate) signal_runtime_custody: crate::source::BridgeSignalRuntimeCustody,
    pub(crate) signal_aspect_lowering_owner: worth_signal::facade::SignalAspectLoweringOwner,
    pub(crate) execution_basis_reservations:
        std::sync::Arc<crate::execution_basis::BridgeExecutionBasisReservationRegistry>,
    pub(crate) correspondence_allocations:
        crate::correspondence::SharedCorrespondenceAllocationRegistry,
    pub(crate) semantic_dependency_registry:
        crate::correspondence::AdmittedSemanticDependencyRegistry,
}

impl std::fmt::Debug for RuntimeBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeBridge")
            .field("policy", &self.policy)
            .field(
                "has_truth_branch_head_source",
                &self.truth_branch_head_source.is_some(),
            )
            .field(
                "has_continuity_lineage_source",
                &self.continuity_lineage_source.is_some(),
            )
            .field(
                "has_writeback_authority",
                &self.writeback_authority.is_some(),
            )
            .field(
                "has_snapshot_reader_pool",
                &self.snapshot_reader_pool.is_some(),
            )
            .field("has_source_adapter", &self.source_adapter.is_some())
            .field(
                "source_contract_count",
                &self.source_registry.contracts().len(),
            )
            .field(
                "structural_contract_count",
                &self.structural_registry.contracts().len(),
            )
            .field(
                "merge_contract_count",
                &self.merge_registry.contracts().len(),
            )
            .field(
                "subscription_family_count",
                &self.subscription_family_registry.registrations().len(),
            )
            .finish()
    }
}

impl RuntimeBridge {
    /// Creates a request-local managed-execution lane over the same installed
    /// truth adapters and mapping authority.
    ///
    /// Signal execution is runtime-affine, so server requests that may execute
    /// on different worker threads must not share one live Signal runtime.
    /// The fork retains installed semantic configuration while minting fresh
    /// Signal ownership and execution-basis reservations.
    pub fn fork_managed_request_lane(&self) -> Self {
        let mut lane = self.clone();
        lane.signal_runtime_key = historical_and_replay::fresh_signal_runtime_key();
        lane.signal_runtime_custody =
            crate::source::BridgeSignalRuntimeCustody::new(lane.signal_runtime_key);
        lane.signal_aspect_lowering_owner =
            worth_signal::facade::SignalAspectLoweringOwner::fresh();
        lane.execution_basis_reservations = Default::default();
        lane
    }

    pub fn bind_signal_graph<'runtime, 'graph>(
        &'runtime self,
        graph: &'graph mut worth_signal::facade::SignalGraph,
    ) -> Result<
        crate::correspondence::BridgeSignalGraphBinding<'runtime, 'graph>,
        crate::correspondence::BridgeCorrespondenceRebindRequired,
    > {
        crate::correspondence::BridgeSignalGraphBinding::admit(self, graph)
    }
}
