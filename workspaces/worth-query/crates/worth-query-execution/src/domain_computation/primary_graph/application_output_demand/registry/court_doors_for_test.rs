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

impl DemandRecord {
    #[doc(hidden)]
    pub fn claim_batch_for_test(&self) -> &Option<claim_readthrough::ClaimBatch> {
        &self.claim_batch
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn prerequisites_for_test(&self) -> &Vec<Arc<WorthQueryOutputDemandKey>> {
        &self.prerequisites
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &DemandState {
        &self.state
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn prerequisites_mut_for_test(&mut self) -> &mut Vec<Arc<WorthQueryOutputDemandKey>> {
        &mut self.prerequisites
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn state_mut_for_test(&mut self) -> &mut DemandState {
        &mut self.state
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn claim_batch_mut_for_test(&mut self) -> &mut Option<claim_readthrough::ClaimBatch> {
        &mut self.claim_batch
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn framework_required_count_for_test(&self) -> &usize {
        &self.framework_required_count
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

impl DemandRecord {
    #[doc(hidden)]
    pub fn source_commits_for_test(&self) -> &Vec<worth_runtime_world::facade::CompositeCommitIdentity> {
        &self.source_commits
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn source_scope_for_test(&self) -> &Option<crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding> {
        &self.source_scope
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn wake_for_test(&self) -> &Arc<DemandWake> {
        &self.wake
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn framework_required_count_mut_for_test(&mut self) -> &mut usize {
        &mut self.framework_required_count
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn source_scope_mut_for_test(&mut self) -> &mut Option<crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding> {
        &mut self.source_scope
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn cleanup_for_test(&self) -> &cleanup::CustodyCleanupLinks {
        &self.cleanup
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn required_keys_for_test(&self) -> &std::collections::BTreeSet<Arc<WorthQueryOutputDemandKey>> {
        &self.required_keys
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn work_membership_for_test(&self) -> &Option<Arc<required_work::RequiredWorkMembership>> {
        &self.work_membership
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn work_membership_mut_for_test(&mut self) -> &mut Option<Arc<required_work::RequiredWorkMembership>> {
        &mut self.work_membership
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn performed_obligations_for_test(&self) -> &Vec<PerformedOutputObligation> {
        &self.performed_obligations
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn performed_source_for_test(&self) -> &Option<WorthQueryPerformedOutputDemandSource> {
        &self.performed_source
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn readmission_source_for_test(&self) -> &Option<Arc<source_readmission::RequiredOutputReadmission>> {
        &self.readmission_source
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn required_stop_for_test(&self) -> &Option<crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind> {
        &self.required_stop
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn successor_of_for_test(&self) -> &Option<succession::Succession> {
        &self.successor_of
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn performed_obligations_mut_for_test(&mut self) -> &mut Vec<PerformedOutputObligation> {
        &mut self.performed_obligations
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn performed_source_mut_for_test(&mut self) -> &mut Option<WorthQueryPerformedOutputDemandSource> {
        &mut self.performed_source
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn readmission_source_mut_for_test(&mut self) -> &mut Option<Arc<source_readmission::RequiredOutputReadmission>> {
        &mut self.readmission_source
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn required_work_queue_mut_for_test(&mut self) -> &mut Option<Arc<required_work::RequiredWorkQueue>> {
        &mut self.required_work_queue
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn source_commits_mut_for_test(&mut self) -> &mut Vec<worth_runtime_world::facade::CompositeCommitIdentity> {
        &mut self.source_commits
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn successor_of_mut_for_test(&mut self) -> &mut Option<succession::Succession> {
        &mut self.successor_of
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn required_keys_mut_for_test(&mut self) -> &mut std::collections::BTreeSet<Arc<WorthQueryOutputDemandKey>> {
        &mut self.required_keys
    }
}

impl BoundOutputSource {
    #[doc(hidden)]
    pub fn new_for_test(scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding, identity: SourceEpoch) -> Self {
        Self { scope, identity }
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn new_for_test(work_membership: Option<Arc<required_work::RequiredWorkMembership>>, interests: usize, required_interests: usize, performed_obligations: Vec<PerformedOutputObligation>, framework_required_count: usize, prerequisites: Vec<Arc<WorthQueryOutputDemandKey>>, claim_batch: Option<claim_readthrough::ClaimBatch>, checkpoint_prerequisites: Option<prerequisite_claims::CheckpointPrerequisiteClaims>, prepared_prerequisite_claims: usize, pending_cleanup_next: Option<Arc<WorthQueryOutputDemandKey>>, pending_cleanup_queued: bool, closed_cleanup: cleanup::ClosedCleanupLinks, settlements: Vec<
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
    >, product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation, source_scope: Option<crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding>, source_commits: Vec<worth_runtime_world::facade::CompositeCommitIdentity>, state: DemandState, performed_source: Option<WorthQueryPerformedOutputDemandSource>, readmission_source: Option<Arc<source_readmission::RequiredOutputReadmission>>, successor_of: Option<succession::Succession>, required_stop: Option<crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>, held_successor: Option<held_successor::HeldRequiredSuccessor>, wake: Arc<DemandWake>) -> Self {
        Self { work_membership, interests, required_interests, performed_obligations, framework_required_count, prerequisites, claim_batch, checkpoint_prerequisites, prepared_prerequisite_claims, pending_cleanup_next, pending_cleanup_queued, closed_cleanup, settlements, product_occurrence, source_scope, source_commits, state, performed_source, readmission_source, successor_of, required_stop, held_successor, wake }
    }
}

impl DemandWake {
    #[doc(hidden)]
    pub fn new_for_test(generation: Mutex<u64>, changed: Condvar) -> Self {
        Self { generation, changed }
    }
}

impl PerformedOutputObligation {
    #[doc(hidden)]
    pub fn new_for_test(source_commit: worth_runtime_world::facade::CompositeCommitIdentity, source: SourceEpoch) -> Self {
        Self { source_commit, source }
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn new_for_test(occurrence: worth_runtime_world::facade::ProductBranchIncarnation, root_kind: PreparedOutputRootKind, source: Option<WorthQueryPerformedOutputDemandSource>, discovery: Option<DiscoveredSourceRecovery>, consumed_sources: Vec<SourceEpoch>, retired_sources: Vec<(SourceEpoch, WorthQueryOutputDemandDenial)>, bound_sources: Option<Vec<BoundOutputSource>>, retired: Option<WorthQueryOutputDemandDenial>, token_count: usize, cleanup: cleanup::CustodyCleanupLinks, completed: bool) -> Self {
        Self { occurrence, root_kind, source, discovery, consumed_sources, retired_sources, bound_sources, retired, token_count, cleanup, completed }
    }
}

impl WorthQueryCompletedOutputDemand {
    #[doc(hidden)]
    pub fn new_for_test(authority: WorthQueryAcceptedOutputAuthority, readiness: super::super::WorthQueryOutputReadinessDeliveryEvidence) -> Self {
        Self { authority, readiness }
    }
}

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn new_for_test(key: WorthQueryOutputDemandKey, requires_output: bool, notifications: WorthQueryOutputDemandNotifications, owner: WorthQueryOutputDemandRegistry) -> Self {
        Self { key, requires_output, notifications, owner }
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
    pub fn new_for_test(receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt, change: Arc<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >, observation: worth_runtime_world::facade::ProductBranchObservation, output_source_identity: Option<SourceEpoch>) -> Self {
        Self { receipt, change, observation, output_source_identity }
    }
}
