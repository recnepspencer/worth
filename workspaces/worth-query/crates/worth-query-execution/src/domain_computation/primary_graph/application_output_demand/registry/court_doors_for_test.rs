//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &WorthQueryOutputDemandKey {
        &self.key
    }
}

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn owner_for_test(&self) -> &WorthQueryOutputDemandRegistry {
        &self.owner
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &Arc<Mutex<DemandRegistryState>> {
        &self.state
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn records_for_test(&self) -> &record_map::DemandRecords {
        &self.records
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn records_mut_for_test(&mut self) -> &mut record_map::DemandRecords {
        &mut self.records
    }
}

impl BoundOutputSource {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &SourceEpoch {
        &self.identity
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn required_work_queue_for_test(&self) -> &Option<Arc<required_work::RequiredWorkQueue>> {
        &self.required_work_queue
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn required_keys_for_test(
        &self,
    ) -> &std::collections::BTreeSet<Arc<WorthQueryOutputDemandKey>> {
        &self.required_keys
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn required_work_queue_mut_for_test(
        &mut self,
    ) -> &mut Option<Arc<required_work::RequiredWorkQueue>> {
        &mut self.required_work_queue
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn required_keys_mut_for_test(
        &mut self,
    ) -> &mut std::collections::BTreeSet<Arc<WorthQueryOutputDemandKey>> {
        &mut self.required_keys
    }
}

impl BoundOutputSource {
    #[doc(hidden)]
    pub fn new_for_test(
        scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
        identity: SourceEpoch,
    ) -> Self {
        Self { scope, identity }
    }
}

impl DemandWake {
    #[doc(hidden)]
    pub fn new_for_test(generation: Mutex<u64>, changed: Condvar) -> Self {
        Self {
            generation,
            changed,
        }
    }
}

impl PerformedOutputObligation {
    #[doc(hidden)]
    pub fn new_for_test(
        source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
        source: SourceEpoch,
    ) -> Self {
        Self {
            source_commit,
            source,
        }
    }
}

impl WorthQueryCompletedOutputDemand {
    #[doc(hidden)]
    pub fn new_for_test(
        authority: WorthQueryAcceptedOutputAuthority,
        readiness: super::super::WorthQueryOutputReadinessDeliveryEvidence,
    ) -> Self {
        Self {
            authority,
            readiness,
        }
    }
}

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn new_for_test(
        key: WorthQueryOutputDemandKey,
        requires_output: bool,
        notifications: WorthQueryOutputDemandNotifications,
        owner: WorthQueryOutputDemandRegistry,
    ) -> Self {
        Self {
            key,
            requires_output,
            notifications,
            owner,
        }
    }
}

impl WorthQueryOutputDemandNotifications {
    #[doc(hidden)]
    pub fn new_for_test(wake: Arc<DemandWake>) -> Self {
        Self { wake }
    }
}

impl WorthQueryPerformedOutputDemandSource {
    #[doc(hidden)]
    pub fn new_for_test(
        receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        change: Arc<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >,
        observation: worth_runtime_world::facade::ProductBranchObservation,
        output_source_identity: Option<SourceEpoch>,
    ) -> Self {
        Self {
            receipt,
            change,
            observation,
            output_source_identity,
        }
    }
}

impl WorthQueryCompletedOutputDemand {
    #[doc(hidden)]
    pub fn authority_for_test(&self) -> &WorthQueryAcceptedOutputAuthority {
        &self.authority
    }
}

impl WorthQueryPerformedOutputDemandSource {
    #[doc(hidden)]
    pub fn change_for_test(&self) -> &Arc<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >{
        &self.change
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn custody_cleanup_for_test(&self) -> &cleanup::CustodyCleanupQueue {
        &self.custody_cleanup
    }
}

impl WorthQueryPerformedOutputDemandSource {
    #[doc(hidden)]
    pub fn receipt_for_test(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
}

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn requires_output_mut_for_test(&mut self) -> &mut bool {
        &mut self.requires_output
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn state_mut_for_test(&mut self) -> &mut Arc<Mutex<DemandRegistryState>> {
        &mut self.state
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn closed_cleanup_for_test(&self) -> &cleanup::ClosedCleanupQueue {
        &self.closed_cleanup
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn source_preparations_for_test(
        &self,
    ) -> &HashMap<worth_runtime_world::facade::ProductBranchIncarnation, SourcePreparationState>
    {
        &self.source_preparations
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn closed_cleanup_mut_for_test(&mut self) -> &mut cleanup::ClosedCleanupQueue {
        &mut self.closed_cleanup
    }
}

impl DemandWake {
    #[doc(hidden)]
    pub fn generation_for_test(&self) -> &Mutex<u64> {
        &self.generation
    }
}

impl PerformedOutputObligation {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &SourceEpoch {
        &self.source
    }
}

impl PerformedOutputObligation {
    #[doc(hidden)]
    pub fn source_commit_for_test(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        &self.source_commit
    }
}

impl BoundOutputSource {
    #[doc(hidden)]
    pub fn identity_into_for_test(self) -> SourceEpoch {
        self.identity
    }
}

impl WorthQueryReadmittedAcceptedOutput {
    #[doc(hidden)]
    pub fn new_for_test(
        checkpoint: WorthQueryAcceptedOutputCheckpointIdentity,
        correspondence: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
        >,
    ) -> Self {
        Self {
            checkpoint,
            correspondence,
        }
    }
}
