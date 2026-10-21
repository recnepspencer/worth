//! Prepare actual demand rows and their source-commit storage.
use super::required_work::{RequiredWorkMembership, RequiredWorkQueue};
use super::{
    DemandRecord, DemandRegistryState, DemandState, DemandWake, WorthQueryOutputDemandKey,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use std::sync::{Arc, Condvar, Mutex};
type Commit = worth_runtime_world::facade::CompositeCommitIdentity;
impl DemandRegistryState {
    pub(super) fn prepare_new_record(
        &self,
        key: &WorthQueryOutputDemandKey,
        source_commit: Option<&Commit>,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        successor_of: Option<[u8; 32]>,
    ) -> Result<PreparedRecord, WorthQueryOutputDemandDenial> {
        let prepared_queue = self
            .required_work_queue
            .is_none()
            .then(|| Arc::new(RequiredWorkQueue::new()));
        let queue = self
            .required_work_queue
            .as_ref()
            .or(prepared_queue.as_ref())
            .expect("prepared queue");
        let work_membership = Arc::new(RequiredWorkMembership::new(Arc::new(key.clone()), queue));
        let closed_cleanup = super::cleanup::ClosedCleanupLinks::new(Arc::clone(&work_membership));
        let mut source_commits = Vec::new();
        if let Some(commit) = source_commit {
            source_commits
                .try_reserve_exact(1)
                .map_err(|_| allocation_denial())?;
            source_commits.push(commit.clone());
        }
        let wake = Arc::new(DemandWake {
            generation: Mutex::new(0),
            changed: Condvar::new(),
        });
        let record = DemandRecord {
            work_membership: Some(work_membership),
            interests: 0,
            required_interests: 0,
            performed_obligations: Vec::new(),
            framework_required_count: 0,
            prerequisites: Vec::new(),
            claim_batch: None,
            checkpoint_prerequisites: None,
            prepared_prerequisite_claims: 0,
            pending_cleanup_next: None,
            pending_cleanup_queued: false,
            closed_cleanup,

            settlements: Vec::new(),
            product_occurrence,
            source_scope: Some(source_scope),
            source_commits,
            state: DemandState::Admitted,
            performed_source: None,
            readmission_source: None,
            successor_of: successor_of.map(super::succession::Succession::new),
            required_stop: None,
            held_successor: None,
            wake,
        };
        Ok(PreparedRecord {
            key: key.clone(),
            record,
            prepared_queue,
        })
    }
    pub(super) fn prepare_source_commit_growth(
        &self,
        record: &DemandRecord,
    ) -> Result<Option<PreparedSourceCommitGrowth>, WorthQueryOutputDemandDenial> {
        self.prepare_source_commit_growth_for(record, 1)
    }
    pub(super) fn prepare_source_commit_growth_for(
        &self,
        record: &DemandRecord,
        additions: usize,
    ) -> Result<Option<PreparedSourceCommitGrowth>, WorthQueryOutputDemandDenial> {
        let needed = record
            .source_commits
            .len()
            .checked_add(additions)
            .ok_or_else(allocation_denial)?;
        if needed <= record.source_commits.capacity() {
            return Ok(None);
        }
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(needed)
            .map_err(|_| allocation_denial())?;
        Ok(Some(PreparedSourceCommitGrowth { slots }))
    }
    pub(super) fn install_prepared_record(&mut self, prepared: PreparedRecord) {
        if let Some(queue) = prepared.prepared_queue {
            debug_assert!(self.required_work_queue.is_none());
            self.required_work_queue = Some(queue);
        }
        self.records.insert(prepared.key, prepared.record);
    }
}
pub(super) struct PreparedRecord {
    pub(super) key: WorthQueryOutputDemandKey,
    pub(super) record: DemandRecord,
    prepared_queue: Option<Arc<RequiredWorkQueue>>,
}
pub(super) struct PreparedSourceCommitGrowth {
    slots: Vec<Commit>,
}
impl PreparedSourceCommitGrowth {
    pub(super) fn install(mut self, record: &mut DemandRecord) {
        self.slots.append(&mut record.source_commits);
        record.source_commits = self.slots;
    }
}
pub(super) fn allocation_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "demand source-commit storage allocation failed",
    )
}
