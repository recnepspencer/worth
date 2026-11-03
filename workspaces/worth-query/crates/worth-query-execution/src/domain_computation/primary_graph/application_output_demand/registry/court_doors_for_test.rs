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
