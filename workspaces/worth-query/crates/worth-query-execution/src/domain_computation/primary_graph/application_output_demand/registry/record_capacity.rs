//! Ordered demand-record storage and its exact final-owner capacity.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use super::{
    required_work::{RequiredWorkMembership, RequiredWorkQueue},
    DemandRecord, DemandRegistryState, DemandState, DemandWake, WorthQueryOutputDemandKey,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

type Commit = worth_runtime_world::facade::CompositeCommitIdentity;

mod navigation;

/// The record and its public wake have distinct final owners. Both debit the
/// registry-record allowance; a notification may retain the wake after the
/// ordered record has been removed.
pub(super) struct RecordCapacity {
    retained: Arc<AtomicUsize>,
    bytes: usize,
    work_membership: Option<std::sync::Weak<RequiredWorkMembership>>,
}

#[cfg(test)]
pub(super) fn fixture_capacity() -> RecordCapacity {
    RecordCapacity {
        retained: Arc::new(AtomicUsize::new(0)),
        bytes: 0,
        work_membership: None,
    }
}

impl Drop for RecordCapacity {
    fn drop(&mut self) {
        if let Some(membership) = self
            .work_membership
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
        {
            membership.set_required(false);
        }
        let previous = self.retained.fetch_sub(self.bytes, Ordering::AcqRel);
        assert!(
            previous >= self.bytes,
            "record capacity has one final owner"
        );
    }
}

impl DemandRegistryState {
    fn reserve_record_capacity(
        &self,
        bytes: usize,
    ) -> Result<RecordCapacity, WorthQueryOutputDemandDenial> {
        let retained = self.record_retained_bytes.load(Ordering::Acquire);
        if retained
            .checked_add(bytes)
            .is_none_or(|total| total > self.record_budget_bytes)
        {
            return Err(capacity_denial());
        }
        self.record_retained_bytes
            .fetch_add(bytes, Ordering::AcqRel);
        Ok(RecordCapacity {
            retained: Arc::clone(&self.record_retained_bytes),
            bytes,
            work_membership: None,
        })
    }

    pub(super) fn prepare_new_record(
        &self,
        key: &WorthQueryOutputDemandKey,
        source_commit: Option<&Commit>,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        successor_of: Option<[u8; 32]>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedRecord, WorthQueryOutputDemandDenial> {
        self.charge_record_lookup(key, admission)?;
        let node_bytes = ordered_node_bytes()?;
        let index_bytes = super::record_map::DemandRecords::source_index_bytes(key)
            .ok_or_else(capacity_denial)?;
        let record_bytes = node_bytes
            .checked_add(index_bytes)
            .and_then(|bytes| bytes.checked_add(key.producer.len()))
            .ok_or_else(capacity_denial)?;
        let root_bytes = self._empty_root_capacity.is_none().then_some(node_bytes);
        let wake_bytes = arc_bytes::<DemandWake>()?;
        let commit_bytes = source_commit
            .map(|_| std::mem::size_of::<Commit>())
            .unwrap_or(0);
        let split_nodes = ordered_levels(self.records.len())?
            .checked_add(1)
            .ok_or_else(capacity_denial)?;
        let split_peak = split_nodes
            .checked_mul(node_bytes)
            .ok_or_else(capacity_denial)?;
        let peak = split_peak
            .checked_add(key.producer.len())
            .and_then(|bytes| bytes.checked_add(index_bytes))
            .and_then(|bytes| bytes.checked_add(wake_bytes))
            .and_then(|bytes| bytes.checked_add(commit_bytes))
            .ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(u64::try_from(peak).map_err(|_| capacity_denial())?)
            .map_err(admission_denial)?;
        let copy_work = key
            .producer
            .len()
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(3))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(copy_work).map_err(|_| work_denial())?)
            .map_err(admission_denial)?;
        let mut record_capacity = self.reserve_record_capacity(record_bytes)?;
        let root_capacity = root_bytes
            .map(|bytes| self.reserve_record_capacity(bytes))
            .transpose()?;
        let wake_capacity = self.reserve_record_capacity(wake_bytes)?;
        let source_commit_capacity = source_commit
            .map(|_| self.reserve_record_capacity(commit_bytes))
            .transpose()?;
        let queue_bytes = self
            .required_work_queue
            .is_none()
            .then(|| {
                arc_bytes::<RequiredWorkQueue>()?
                    .checked_add(arc_bytes::<RecordCapacity>()?)
                    .ok_or_else(capacity_denial)
            })
            .transpose()?;
        let membership_bytes = arc_bytes::<RequiredWorkMembership>()?
            .checked_add(arc_bytes::<WorthQueryOutputDemandKey>()?)
            .and_then(|bytes| bytes.checked_add(key.producer.len()))
            .ok_or_else(capacity_denial)?;
        let work_backing = queue_bytes
            .unwrap_or(0)
            .checked_add(membership_bytes)
            .ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(u64::try_from(work_backing).map_err(|_| capacity_denial())?)
            .map_err(admission_denial)?;
        admission
            .charge_external_work(
                u64::try_from(key.producer.len().checked_add(3).ok_or_else(work_denial)?)
                    .map_err(|_| work_denial())?,
            )
            .map_err(admission_denial)?;
        let prepared_queue = queue_bytes
            .map(|bytes| {
                self.reserve_record_capacity(bytes)
                    .map(|capacity| Arc::new(RequiredWorkQueue::new(Arc::new(capacity))))
            })
            .transpose()?;
        let queue = self
            .required_work_queue
            .as_ref()
            .or(prepared_queue.as_ref())
            .expect("a new demand retains its required-work queue");
        let membership_capacity = self.reserve_record_capacity(membership_bytes)?;
        let work_membership = Arc::new(RequiredWorkMembership::new(
            Arc::new(key.clone()),
            queue,
            membership_capacity,
        ));
        record_capacity.work_membership = Some(Arc::downgrade(&work_membership));
        let mut source_commits = Vec::new();
        if source_commit.is_some() {
            source_commits
                .try_reserve_exact(1)
                .map_err(|_| capacity_denial())?;
            if source_commits.capacity() > 1 {
                return Err(capacity_denial());
            }
        }
        if let Some(commit) = source_commit {
            source_commits.push(commit.clone());
        }
        let wake = Arc::new(DemandWake {
            generation: Mutex::new(0),
            changed: Condvar::new(),
            _record_capacity: wake_capacity,
        });
        let record = DemandRecord {
            _record_capacity: record_capacity,
            work_membership: Some(work_membership),
            interests: 0,
            required_interests: 0,
            performed_obligations: Vec::new(),
            framework_required_count: 0,
            prerequisites: Vec::new(),
            checkpoint_prerequisites: None,
            prepared_prerequisite_claims: 0,
            pending_cleanup_next: None,
            pending_cleanup_queued: false,
            pending_cleanup_key_bytes: 0,
            settlements: Vec::new(),
            product_occurrence,
            source_scope: Some(source_scope),
            source_commits,
            source_commit_capacity,
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
            root_capacity,
            prepared_queue,
        })
    }

    pub(super) fn prepare_source_commit_growth(
        &self,
        record: &DemandRecord,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<PreparedSourceCommitGrowth>, WorthQueryOutputDemandDenial> {
        self.prepare_source_commit_growth_for(record, 1, admission)
    }

    pub(super) fn prepare_source_commit_growth_for(
        &self,
        record: &DemandRecord,
        additions: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<PreparedSourceCommitGrowth>, WorthQueryOutputDemandDenial> {
        debug_assert_eq!(
            record.source_commits.capacity() == 0,
            record.source_commit_capacity.is_none()
        );
        let needed = record
            .source_commits
            .len()
            .checked_add(additions)
            .ok_or_else(capacity_denial)?;
        admission
            .charge_external_work(u64::try_from(needed).map_err(|_| work_denial())?)
            .map_err(admission_denial)?;
        if needed <= record.source_commits.capacity() {
            return Ok(None);
        }
        let bytes = needed
            .checked_mul(std::mem::size_of::<Commit>())
            .ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(u64::try_from(bytes).map_err(|_| capacity_denial())?)
            .map_err(admission_denial)?;
        let capacity = self.reserve_record_capacity(bytes)?;
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(needed)
            .map_err(|_| capacity_denial())?;
        if slots.capacity() > needed {
            return Err(capacity_denial());
        }
        Ok(Some(PreparedSourceCommitGrowth { slots, capacity }))
    }
}

pub(super) struct PreparedRecord {
    pub(super) key: WorthQueryOutputDemandKey,
    pub(super) record: DemandRecord,
    root_capacity: Option<RecordCapacity>,
    prepared_queue: Option<Arc<RequiredWorkQueue>>,
}

impl DemandRegistryState {
    pub(super) fn install_prepared_record(&mut self, prepared: PreparedRecord) {
        if let Some(queue) = prepared.prepared_queue {
            debug_assert!(self.required_work_queue.is_none());
            self.required_work_queue = Some(queue);
        }
        if let Some(root_capacity) = prepared.root_capacity {
            debug_assert!(self._empty_root_capacity.is_none());
            self._empty_root_capacity = Some(root_capacity);
        }
        self.records.insert(prepared.key, prepared.record);
    }
}

fn ordered_levels(count: usize) -> Result<usize, WorthQueryOutputDemandDenial> {
    let mut levels = 1usize;
    let mut minimum_next_level = 11usize;
    while count >= minimum_next_level {
        levels = levels.checked_add(1).ok_or_else(work_denial)?;
        minimum_next_level = minimum_next_level
            .checked_add(1)
            .and_then(|next| next.checked_mul(6))
            .and_then(|next| next.checked_sub(1))
            .unwrap_or(usize::MAX);
        if minimum_next_level == usize::MAX {
            break;
        }
    }
    Ok(levels)
}

pub(super) struct PreparedSourceCommitGrowth {
    slots: Vec<Commit>,
    capacity: RecordCapacity,
}

impl PreparedSourceCommitGrowth {
    pub(super) fn install(self, record: &mut DemandRecord) {
        let old_slots = std::mem::replace(&mut record.source_commits, self.slots);
        let old_capacity = record.source_commit_capacity.replace(self.capacity);
        record.source_commits.extend(old_slots);
        drop(old_capacity);
    }
}

fn ordered_node_bytes() -> Result<usize, WorthQueryOutputDemandDenial> {
    12usize
        .checked_mul(std::mem::size_of::<(WorthQueryOutputDemandKey, DemandRecord)>())
        .and_then(|bytes| bytes.checked_add(14 * std::mem::size_of::<usize>()))
        .ok_or_else(capacity_denial)
}

fn arc_bytes<T>() -> Result<usize, WorthQueryOutputDemandDenial> {
    let (layout, _) = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map_err(|_| capacity_denial())?;
    Ok(layout.pad_to_align().size())
}

fn admission_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow => work_denial(),
        _ => capacity_denial(),
    }
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "ordered demand-record navigation exceeds request work",
    )
}

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "ordered demand-record storage exceeds retained capacity",
    )
}

impl super::WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn with_budgets(
        obligation_bytes: usize,
        record_bytes: usize,
        required_bytes: usize,
    ) -> Self {
        let state = DemandRegistryState {
            obligation_budget_bytes: obligation_bytes,
            record_budget_bytes: record_bytes,
            required_budget_bytes: required_bytes,
            ..DemandRegistryState::default()
        };
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }
}
