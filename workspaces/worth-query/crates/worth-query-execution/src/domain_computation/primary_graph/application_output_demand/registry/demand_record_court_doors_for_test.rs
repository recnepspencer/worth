//! Sealed test doors over a demand record's private members.
use super::*;

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

impl DemandRecord {
    #[doc(hidden)]
    pub fn source_commits_for_test(
        &self,
    ) -> &Vec<worth_runtime_world::facade::CompositeCommitIdentity> {
        &self.source_commits
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn source_scope_for_test(
        &self,
    ) -> &Option<crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding>
    {
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
    pub fn source_scope_mut_for_test(
        &mut self,
    ) -> &mut Option<crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding>
    {
        &mut self.source_scope
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
    pub fn work_membership_mut_for_test(
        &mut self,
    ) -> &mut Option<Arc<required_work::RequiredWorkMembership>> {
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
    pub fn readmission_source_for_test(
        &self,
    ) -> &Option<Arc<source_readmission::RequiredOutputReadmission>> {
        &self.readmission_source
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn required_stop_for_test(
        &self,
    ) -> &Option<crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind> {
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
    pub fn performed_source_mut_for_test(
        &mut self,
    ) -> &mut Option<WorthQueryPerformedOutputDemandSource> {
        &mut self.performed_source
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn readmission_source_mut_for_test(
        &mut self,
    ) -> &mut Option<Arc<source_readmission::RequiredOutputReadmission>> {
        &mut self.readmission_source
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn source_commits_mut_for_test(
        &mut self,
    ) -> &mut Vec<worth_runtime_world::facade::CompositeCommitIdentity> {
        &mut self.source_commits
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn successor_of_mut_for_test(&mut self) -> &mut Option<succession::Succession> {
        &mut self.successor_of
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn new_for_test(
        work_membership: Option<Arc<required_work::RequiredWorkMembership>>,
        interests: usize,
        required_interests: usize,
        performed_obligations: Vec<PerformedOutputObligation>,
        framework_required_count: usize,
        prerequisites: Vec<Arc<WorthQueryOutputDemandKey>>,
        claim_batch: Option<claim_readthrough::ClaimBatch>,
        checkpoint_prerequisites: Option<prerequisite_claims::CheckpointPrerequisiteClaims>,
        prepared_prerequisite_claims: usize,
        pending_cleanup_next: Option<Arc<WorthQueryOutputDemandKey>>,
        pending_cleanup_queued: bool,
        closed_cleanup: cleanup::ClosedCleanupLinks,
        settlements: Vec<
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
    >,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        source_scope: Option<
            crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        >,
        source_commits: Vec<worth_runtime_world::facade::CompositeCommitIdentity>,
        state: DemandState,
        performed_source: Option<WorthQueryPerformedOutputDemandSource>,
        readmission_source: Option<Arc<source_readmission::RequiredOutputReadmission>>,
        successor_of: Option<succession::Succession>,
        required_stop: Option<
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind,
        >,
        held_successor: Option<held_successor::HeldRequiredSuccessor>,
        wake: Arc<DemandWake>,
    ) -> Self {
        Self {
            work_membership,
            interests,
            required_interests,
            performed_obligations,
            framework_required_count,
            prerequisites,
            claim_batch,
            checkpoint_prerequisites,
            prepared_prerequisite_claims,
            pending_cleanup_next,
            pending_cleanup_queued,
            closed_cleanup,
            settlements,
            product_occurrence,
            source_scope,
            source_commits,
            state,
            performed_source,
            readmission_source,
            successor_of,
            required_stop,
            held_successor,
            wake,
        }
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn interests_for_test(&self) -> &usize {
        &self.interests
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn settlements_for_test(
        &self,
    ) -> &Vec<
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
    > {
        &self.settlements
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn required_interests_mut_for_test(&mut self) -> &mut usize {
        &mut self.required_interests
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn required_stop_mut_for_test(
        &mut self,
    ) -> &mut Option<crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>
    {
        &mut self.required_stop
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn pending_cleanup_queued_for_test(&self) -> &bool {
        &self.pending_cleanup_queued
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn interests_mut_for_test(&mut self) -> &mut usize {
        &mut self.interests
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn closed_cleanup_for_test(&self) -> &cleanup::ClosedCleanupLinks {
        &self.closed_cleanup
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn held_successor_for_test(&self) -> &Option<held_successor::HeldRequiredSuccessor> {
        &self.held_successor
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn required_interests_for_test(&self) -> &usize {
        &self.required_interests
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn settlements_mut_for_test(
        &mut self,
    ) -> &mut Vec<
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
    > {
        &mut self.settlements
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn wake_mut_for_test(&mut self) -> &mut Arc<DemandWake> {
        &mut self.wake
    }
}
