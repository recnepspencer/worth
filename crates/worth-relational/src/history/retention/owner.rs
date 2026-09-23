use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::branch::{RelationalBranchIdentity, RelationalBranchRoot};

use super::cost_counters::{RelationalRetentionAtomicCounters, RelationalRetentionCostCounters};

const MAX_LIVE_ROOT_OBLIGATIONS: usize = 65_536;
// Exact history consumers can retain a root for each of 10k publications.
// Keep this owner inventory finite while admitting that supported history.
const MAX_RETIRED_BRANCH_ROOTS: usize = 16_384;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalRetentionAcquisitionDenial {
    OwnerUnavailable,
    CapacityExhausted,
    IdentityExhausted,
    RootSetTooLarge,
}

#[derive(Debug, Clone)]
pub(crate) struct RelationalBranchRetentionOwner {
    pub(super) inner: Arc<RelationalBranchRetentionOwnerInner>,
}

#[derive(Debug)]
pub(super) struct RelationalBranchRetentionOwnerInner {
    pub(super) runtime_instance_id: u64,
    max_live_root_obligations: usize,
    max_retired_branch_roots: usize,
    pub(super) live_head_count: AtomicUsize,
    live_retention_count: AtomicUsize,
    pub(super) ordinary_counters: RelationalRetentionAtomicCounters,
    pub(super) head_install_count: AtomicU64,
    pub(super) head_transfer_count: AtomicU64,
    pub(super) next_retirement_generation: AtomicU64,
    pub(super) retirement_slot_count: AtomicUsize,
    pub(super) retired_root_count: AtomicUsize,
    pub(super) retired_roots: DashMap<usize, RelationalRetiredBranchRoot>,
    pub(super) retired_roots_by_commit: DashMap<crate::history::data::CommitId, (usize, u64)>,
    pub(super) retired_root_order: SegQueue<(usize, u64)>,
    pub(super) retired_enqueue_count: AtomicU64,
    pub(super) maintenance_counters: Mutex<RelationalRetentionCostCounters>,
}

#[derive(Debug)]
pub(super) struct RelationalRetiredBranchRoot {
    pub(super) root: Option<Arc<RelationalBranchRoot>>,
    pub(super) reservations: usize,
    pub(super) retired: bool,
    pub(super) generation: u64,
}

impl RelationalBranchRetentionOwner {
    pub(crate) fn new(runtime_instance_id: u64) -> Self {
        Self::new_with_limits(
            runtime_instance_id,
            MAX_LIVE_ROOT_OBLIGATIONS,
            MAX_RETIRED_BRANCH_ROOTS,
        )
    }

    fn new_with_limits(
        runtime_instance_id: u64,
        max_live_root_obligations: usize,
        max_retired_branch_roots: usize,
    ) -> Self {
        Self {
            inner: Arc::new(RelationalBranchRetentionOwnerInner {
                runtime_instance_id,
                max_live_root_obligations,
                max_retired_branch_roots,
                live_head_count: AtomicUsize::new(0),
                live_retention_count: AtomicUsize::new(0),
                ordinary_counters: RelationalRetentionAtomicCounters::default(),
                head_install_count: AtomicU64::new(0),
                head_transfer_count: AtomicU64::new(0),
                next_retirement_generation: AtomicU64::new(0),
                retirement_slot_count: AtomicUsize::new(0),
                retired_root_count: AtomicUsize::new(0),
                retired_roots: DashMap::new(),
                retired_roots_by_commit: DashMap::new(),
                retired_root_order: SegQueue::new(),
                retired_enqueue_count: AtomicU64::new(0),
                maintenance_counters: Mutex::new(RelationalRetentionCostCounters::default()),
            }),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_limits(
        runtime_instance_id: u64,
        max_live_root_obligations: usize,
        max_retired_branch_roots: usize,
    ) -> Self {
        Self::new_with_limits(
            runtime_instance_id,
            max_live_root_obligations,
            max_retired_branch_roots,
        )
    }

    pub(crate) fn binding(&self) -> super::RelationalBranchRetentionBinding {
        super::RelationalBranchRetentionBinding::new(&self.inner, None)
    }

    pub(crate) fn install_head(
        &self,
        identity: RelationalBranchIdentity,
        root: &Arc<RelationalBranchRoot>,
    ) -> Result<super::RelationalHeadRetentionObligation, RelationalRetentionAcquisitionDenial>
    {
        if identity.runtime_instance_id() != self.inner.runtime_instance_id {
            return Err(RelationalRetentionAcquisitionDenial::OwnerUnavailable);
        }
        reserve_live_retention_capacity(&self.inner)?;
        self.inner.live_head_count.fetch_add(1, Ordering::Relaxed);
        self.inner
            .head_install_count
            .fetch_add(1, Ordering::Relaxed);
        Ok(super::RelationalHeadRetentionObligation::new(
            &self.inner,
            identity,
            Arc::clone(root),
        ))
    }

    pub(crate) fn counters(&self) -> RelationalRetentionCostCounters {
        let mut counters = lock_maintenance_counters(&self.inner)
            .to_owned()
            .saturating_add(self.inner.ordinary_counters.snapshot());
        counters.head_installs = self.inner.head_install_count.load(Ordering::Relaxed);
        counters.head_transfers = self.inner.head_transfer_count.load(Ordering::Relaxed);
        counters.retired_root_enqueues = self.inner.retired_enqueue_count.load(Ordering::Relaxed);
        counters
    }

    pub(crate) fn acquire_retired_observation(
        &self,
        commit_id: crate::history::data::CommitId,
    ) -> Result<
        Option<(Arc<RelationalBranchRoot>, super::RelationalRetentionGuard)>,
        RelationalRetentionAcquisitionDenial,
    > {
        let Some(root) = self
            .inner
            .retired_roots_by_commit
            .get(&commit_id)
            .and_then(|indexed| {
                let (root_key, generation) = *indexed.value();
                self.inner
                    .retired_roots
                    .get(&root_key)
                    .filter(|retired| retired.generation == generation)
            })
            .and_then(|retired| retired.root.as_ref().map(Arc::clone))
        else {
            return Ok(None);
        };
        let guard = self.binding().acquire(
            super::RelationalRetentionObligationKind::Observation,
            vec![Arc::clone(&root)],
            None,
        )?;
        Ok(Some((root, guard)))
    }
}

pub(super) fn lock_maintenance_counters(
    owner: &RelationalBranchRetentionOwnerInner,
) -> std::sync::MutexGuard<'_, RelationalRetentionCostCounters> {
    owner
        .maintenance_counters
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn reserve_live_obligation_capacity(
    owner: &RelationalBranchRetentionOwnerInner,
) -> Result<(), RelationalRetentionAcquisitionDenial> {
    reserve_live_retention_capacity(owner)
}

pub(super) fn release_live_obligation_capacity(owner: &RelationalBranchRetentionOwnerInner) {
    owner.live_retention_count.fetch_sub(1, Ordering::Release);
}

pub(super) fn release_live_head_capacity(owner: &RelationalBranchRetentionOwnerInner) {
    owner.live_head_count.fetch_sub(1, Ordering::Relaxed);
    owner.live_retention_count.fetch_sub(1, Ordering::Release);
}

fn reserve_live_retention_capacity(
    owner: &RelationalBranchRetentionOwnerInner,
) -> Result<(), RelationalRetentionAcquisitionDenial> {
    let mut current = owner.live_retention_count.load(Ordering::Acquire);
    loop {
        if current >= owner.max_live_root_obligations {
            return Err(RelationalRetentionAcquisitionDenial::CapacityExhausted);
        }
        match owner.live_retention_count.compare_exchange_weak(
            current,
            current + 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return Ok(()),
            Err(observed) => current = observed,
        }
    }
}

pub(super) fn reserve_live_head_capacity(
    owner: &RelationalBranchRetentionOwnerInner,
) -> Result<(), RelationalRetentionAcquisitionDenial> {
    reserve_live_retention_capacity(owner)
}

pub(super) fn reserve_retirement_slot(
    owner: &RelationalBranchRetentionOwnerInner,
) -> Result<u64, RelationalRetentionAcquisitionDenial> {
    let mut current = owner.retirement_slot_count.load(Ordering::Acquire);
    loop {
        if current >= owner.max_retired_branch_roots {
            return Err(RelationalRetentionAcquisitionDenial::CapacityExhausted);
        }
        match owner.retirement_slot_count.compare_exchange_weak(
            current,
            current + 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => break,
            Err(observed) => current = observed,
        }
    }
    match owner.next_retirement_generation.fetch_update(
        Ordering::AcqRel,
        Ordering::Acquire,
        |generation| generation.checked_add(1),
    ) {
        Ok(previous) => Ok(previous + 1),
        Err(_) => {
            release_retirement_slot(owner);
            Err(RelationalRetentionAcquisitionDenial::IdentityExhausted)
        }
    }
}

pub(super) fn release_retirement_slot(owner: &RelationalBranchRetentionOwnerInner) {
    owner.retirement_slot_count.fetch_sub(1, Ordering::Release);
}
