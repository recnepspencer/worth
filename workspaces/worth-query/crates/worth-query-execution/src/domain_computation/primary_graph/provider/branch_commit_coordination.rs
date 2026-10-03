//! Exact product-branch coordination for application commit progression.

use std::collections::BTreeMap;
use std::mem::{align_of, size_of};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::{ProductBranchIncarnation, ProductBranchObservation};

use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

pub(super) struct WorthQueryApplicationBranchCommitCoordinator {
    lanes: Mutex<BTreeMap<ProductBranchIncarnation, LaneEntry>>,
    capacity: Arc<Mutex<CoordinationCapacity>>,
}

pub(crate) struct WorthQueryApplicationBranchCommitLane {
    occurrence: ProductBranchIncarnation,
    transition: Mutex<()>,
    _ticket: Arc<LaneTicket>,
}

struct LaneEntry {
    weak: Weak<WorthQueryApplicationBranchCommitLane>,
    _ticket: Arc<LaneTicket>,
}

struct CoordinationCapacity {
    maximum: usize,
    retained: usize,
}

struct LaneTicket {
    capacity: Arc<Mutex<CoordinationCapacity>>,
    bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorthQueryBranchCommitLaneDenial {
    RetainedCapacityExhausted {
        requested: usize,
        retained: usize,
        maximum: usize,
    },
    CapacityCounterOverflow,
    Preparation(CompanionPreflightStop),
}

impl From<CompanionPreflightStop> for WorthQueryBranchCommitLaneDenial {
    fn from(stop: CompanionPreflightStop) -> Self {
        Self::Preparation(stop)
    }
}

enum LaneAdmission<'a> {
    Ordinary,
    Metered(&'a mut InvalidationEditAdmission),
}

impl LaneAdmission<'_> {
    fn work(&mut self, visits: u64) -> Result<(), WorthQueryBranchCommitLaneDenial> {
        match self {
            Self::Ordinary => Ok(()),
            Self::Metered(admission) => admission
                .charge_external_work(visits)
                .map_err(WorthQueryBranchCommitLaneDenial::Preparation),
        }
    }

    fn bytes(&mut self, bytes: u64) -> Result<(), WorthQueryBranchCommitLaneDenial> {
        match self {
            Self::Ordinary => Ok(()),
            Self::Metered(admission) => admission
                .admit_read_scratch(bytes)
                .map_err(WorthQueryBranchCommitLaneDenial::Preparation),
        }
    }
}

pub(crate) struct WorthQueryApplicationBranchCommitCoordination<'lane> {
    occurrence: ProductBranchIncarnation,
    _guard: MutexGuard<'lane, ()>,
}

impl WorthQueryApplicationBranchCommitCoordinator {
    pub(super) fn new(maximum_retained_bytes: usize) -> Self {
        Self {
            lanes: Mutex::new(BTreeMap::new()),
            capacity: Arc::new(Mutex::new(CoordinationCapacity {
                maximum: maximum_retained_bytes,
                retained: 0,
            })),
        }
    }

    /// The stable-output path pays for the same lane lookup and construction
    /// before it holds the branch transition across publication preparation.
    pub(super) fn lane_for_admitted(
        &self,
        observation: &ProductBranchObservation,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Arc<WorthQueryApplicationBranchCommitLane>, WorthQueryBranchCommitLaneDenial> {
        self.lane_for_with(
            observation.lifecycle_incarnation(),
            LaneAdmission::Metered(admission),
        )
    }

    pub(super) fn lane_for(
        &self,
        observation: &ProductBranchObservation,
    ) -> Result<Arc<WorthQueryApplicationBranchCommitLane>, WorthQueryBranchCommitLaneDenial> {
        self.lane_for_occurrence(observation.lifecycle_incarnation())
    }

    pub(super) fn lane_for_occurrence(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Result<Arc<WorthQueryApplicationBranchCommitLane>, WorthQueryBranchCommitLaneDenial> {
        self.lane_for_with(occurrence, LaneAdmission::Ordinary)
    }

    fn lane_for_with(
        &self,
        occurrence: ProductBranchIncarnation,
        mut admission: LaneAdmission<'_>,
    ) -> Result<Arc<WorthQueryApplicationBranchCommitLane>, WorthQueryBranchCommitLaneDenial> {
        let mut lanes = self
            .lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entries = lanes.len();
        admission.work(tree_navigation_work(entries)?)?;
        let prior = lanes.get(&occurrence);
        if prior.is_some() {
            admission.work(1)?;
        }
        if let Some(lane) = prior.and_then(|entry| entry.weak.upgrade()) {
            return Ok(lane);
        }

        admission.work(tree_navigation_work(entries)?.checked_add(2).ok_or(
            WorthQueryBranchCommitLaneDenial::Preparation(
                CompanionPreflightStop::WorkCounterOverflow,
            ),
        )?)?;
        // An expired Weak still owns the old Arc control block and the tree
        // entry. Reuse its ticket for the replacement lane; its admission
        // already retains the persistent entry/path. Only a genuinely absent
        // entry reserves a new retained ticket.
        let prior_ticket = prior.map(|entry| Arc::clone(&entry._ticket));
        let mut bytes = arc_bytes::<WorthQueryApplicationBranchCommitLane>()?;
        if prior_ticket.is_none() {
            let path = tree_insert_bytes::<ProductBranchIncarnation, LaneEntry>(entries)?;
            bytes = bytes
                .checked_add(arc_bytes::<LaneTicket>()?)
                .and_then(|bytes| bytes.checked_add(path))
                .ok_or(WorthQueryBranchCommitLaneDenial::CapacityCounterOverflow)?;
        }
        admission.bytes(u64::try_from(bytes).map_err(|_| {
            WorthQueryBranchCommitLaneDenial::Preparation(
                CompanionPreflightStop::PreparationMemoryCounterOverflow,
            )
        })?)?;
        let ticket = match prior_ticket {
            Some(ticket) => ticket,
            None => self.reserve(bytes)?,
        };
        let lane = Arc::new(WorthQueryApplicationBranchCommitLane {
            occurrence,
            transition: Mutex::new(()),
            _ticket: Arc::clone(&ticket),
        });
        let replaced = lanes.insert(
            occurrence,
            LaneEntry {
                weak: Arc::downgrade(&lane),
                _ticket: ticket,
            },
        );
        drop(lanes);
        drop(replaced);
        Ok(lane)
    }

    fn reserve(&self, bytes: usize) -> Result<Arc<LaneTicket>, WorthQueryBranchCommitLaneDenial> {
        let mut capacity = self
            .capacity
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let requested = capacity
            .retained
            .checked_add(bytes)
            .ok_or(WorthQueryBranchCommitLaneDenial::CapacityCounterOverflow)?;
        if requested > capacity.maximum {
            return Err(
                WorthQueryBranchCommitLaneDenial::RetainedCapacityExhausted {
                    requested,
                    retained: capacity.retained,
                    maximum: capacity.maximum,
                },
            );
        }
        capacity.retained = requested;
        Ok(Arc::new(LaneTicket {
            capacity: Arc::clone(&self.capacity),
            bytes,
        }))
    }

    pub(super) fn retire(&self, occurrence: ProductBranchIncarnation) {
        let (retired, empty_root) = {
            let mut lanes = self
                .lanes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let retired = lanes.remove(&occurrence);
            // std's B-tree may retain an empty root allocation. Move it out
            // before the last entry ticket can refund that root's custody.
            let empty_root = lanes.is_empty().then(|| std::mem::take(&mut *lanes));
            (retired, empty_root)
        };
        drop(empty_root);
        drop(retired);
    }

    #[cfg(test)]
    fn retained_bytes(&self) -> usize {
        self.capacity
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retained
    }

    #[cfg(test)]
    fn entry_count(&self) -> usize {
        self.lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
}

impl Drop for LaneTicket {
    fn drop(&mut self) {
        let mut capacity = self
            .capacity
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        capacity.retained = capacity
            .retained
            .checked_sub(self.bytes)
            .expect("the branch lane ticket refunds its exact retained bytes");
    }
}

fn tree_navigation_work(entries: usize) -> Result<u64, CompanionPreflightStop> {
    // std's B-tree has at most eleven initialized keys per node. A nonroot
    // node has at least five, so a tree with another level needs at least
    // 11, then 71, then 431 entries. An empty root still costs one visit.
    if entries == 0 {
        return Ok(1);
    }
    let mut levels = 1usize;
    let mut next_minimum = 11usize;
    while entries >= next_minimum {
        levels = levels
            .checked_add(1)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        let Some(next) = next_minimum
            .checked_add(1)
            .and_then(|minimum| minimum.checked_mul(6))
            .and_then(|minimum| minimum.checked_sub(1))
        else {
            break;
        };
        next_minimum = next;
    }
    let comparisons = entries
        .min(11)
        .checked_mul(levels)
        .and_then(|visits| visits.checked_add(1))
        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    u64::try_from(comparisons).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)
}

fn tree_insert_bytes<K, V>(entries: usize) -> Result<usize, WorthQueryBranchCommitLaneDenial> {
    // A split can allocate one new node per occupied level and a new root.
    // The bit-length height is conservative for every std B-tree posture.
    let levels = usize::BITS as usize - entries.max(1).leading_zeros() as usize;
    let node = size_of::<(K, V)>()
        .checked_mul(11)
        .and_then(|bytes| bytes.checked_add(size_of::<usize>().checked_mul(16)?))
        .and_then(|bytes| bytes.checked_add(64))
        .ok_or(WorthQueryBranchCommitLaneDenial::CapacityCounterOverflow)?;
    let bytes = node
        .checked_mul(
            levels
                .checked_add(2)
                .ok_or(WorthQueryBranchCommitLaneDenial::CapacityCounterOverflow)?,
        )
        .ok_or(WorthQueryBranchCommitLaneDenial::CapacityCounterOverflow)?;
    Ok(bytes)
}

fn arc_bytes<T>() -> Result<usize, WorthQueryBranchCommitLaneDenial> {
    let alignment = align_of::<T>().max(align_of::<usize>());
    let header = size_of::<usize>()
        .checked_mul(2)
        .ok_or(WorthQueryBranchCommitLaneDenial::CapacityCounterOverflow)?;
    let body_offset = header
        .checked_add(alignment - 1)
        .map(|bytes| bytes / alignment * alignment)
        .ok_or(WorthQueryBranchCommitLaneDenial::CapacityCounterOverflow)?;
    let bytes = body_offset
        .checked_add(size_of::<T>())
        .and_then(|bytes| bytes.checked_add(alignment - 1))
        .map(|bytes| bytes / alignment * alignment)
        .ok_or(WorthQueryBranchCommitLaneDenial::CapacityCounterOverflow)?;
    Ok(bytes)
}

impl WorthQueryApplicationBranchCommitLane {
    pub(crate) fn enter(&self) -> WorthQueryApplicationBranchCommitCoordination<'_> {
        WorthQueryApplicationBranchCommitCoordination {
            occurrence: self.occurrence,
            _guard: self
                .transition
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        }
    }
}

impl WorthQueryApplicationBranchCommitCoordination<'_> {
    pub(in crate::domain_computation) fn admits(
        &self,
        observation: &ProductBranchObservation,
    ) -> bool {
        self.occurrence == observation.lifecycle_incarnation()
    }
}

#[cfg(test)]
mod tests;
