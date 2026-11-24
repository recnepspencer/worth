//! Exact-prediction reference state machine for `PhysicalResidencyPool`.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use worth_store_buffer_pool::{
    DirtyPhysicalFrame, PhysicalFrameLease, PhysicalResidencyDenial, PhysicalResidencyPool,
};

pub(super) const PAGE_COUNT: u64 = 7;
pub(super) const POOL_CAPACITY: usize = 4;

pub(super) struct ModelState {
    pub(super) backing: BTreeMap<u64, [u8; 8]>,
    pub(super) resident: BTreeSet<u64>,
    pub(super) dirty: BTreeMap<u64, [u8; 8]>,
    pub(super) evictable: VecDeque<u64>,
    pub(super) clean_leases: Vec<(u64, PhysicalFrameLease)>,
    pub(super) dirty_handles: Vec<(u64, DirtyPhysicalFrame)>,
}

impl ModelState {
    pub(super) fn new() -> Self {
        let mut backing = BTreeMap::new();
        for p in 1..=PAGE_COUNT {
            backing.insert(p, [p as u8; 8]);
        }
        Self {
            backing,
            resident: BTreeSet::new(),
            dirty: BTreeMap::new(),
            evictable: VecDeque::new(),
            clean_leases: Vec::new(),
            dirty_handles: Vec::new(),
        }
    }

    pub(super) fn expected_bytes(&self, page: u64) -> [u8; 8] {
        self.dirty
            .get(&page)
            .copied()
            .unwrap_or_else(|| self.backing[&page])
    }

    pub(super) fn pin_count(&self, page: u64) -> usize {
        self.clean_leases.iter().filter(|(p, _)| *p == page).count()
            + self
                .dirty_handles
                .iter()
                .filter(|(p, _)| *p == page)
                .count()
    }

    pub(super) fn total_leases(&self) -> usize {
        self.clean_leases.len() + self.dirty_handles.len()
    }

    pub(super) fn pinned_frames(&self) -> usize {
        self.resident
            .iter()
            .filter(|&&p| self.pin_count(p) > 0)
            .count()
    }

    pub(super) fn fixed_frames(&self) -> usize {
        self.resident.len() - self.evictable.len()
    }

    pub(super) fn detach_evictable(&mut self, page: u64) {
        if let Some(pos) = self.evictable.iter().position(|&p| p == page) {
            self.evictable.remove(pos);
        }
    }

    pub(super) fn maybe_mark_evictable(&mut self, page: u64) {
        if self.resident.contains(&page)
            && self.pin_count(page) == 0
            && !self.dirty.contains_key(&page)
            && !self.evictable.contains(&page)
        {
            self.evictable.push_back(page);
        }
    }

    pub(super) fn admit_new_resident(&mut self, page: u64) {
        if self.resident.len() == POOL_CAPACITY {
            let victim = self
                .evictable
                .pop_front()
                .expect("pool at capacity must evict oldest clean unpinned frame");
            assert!(self.resident.remove(&victim));
        }
        assert!(self.resident.insert(page));
    }

    pub(super) fn candidate_denial(&self, page: u64) -> Option<CandidateDenialKind> {
        if self.resident.contains(&page) {
            return Some(CandidateDenialKind::AlreadyResident);
        }
        if self.fixed_frames() >= POOL_CAPACITY
            || self.dirty.len() >= POOL_CAPACITY
            || self.pinned_frames() >= POOL_CAPACITY
            || self.total_leases() >= POOL_CAPACITY
        {
            return Some(CandidateDenialKind::Pressure);
        }
        None
    }

    pub(super) fn assert_invariants(&self, pool: &PhysicalResidencyPool) {
        let c = pool.counters();
        assert_eq!(
            c.frame_entries(),
            self.resident.len() as u32,
            "frame_entries"
        );
        assert_eq!(
            c.pinned_frames(),
            self.pinned_frames() as u32,
            "pinned_frames"
        );
        assert_eq!(c.pin_leases(), self.total_leases() as u32, "pin_leases");
        assert_eq!(c.dirty_frames(), self.dirty.len() as u32, "dirty_frames");
        assert_eq!(c.active_writeback_claims(), 0, "no leaked writeback claims");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CandidateDenialKind {
    AlreadyResident,
    Pressure,
}

impl CandidateDenialKind {
    pub(super) fn matches(self, err: PhysicalResidencyDenial) -> bool {
        match self {
            Self::AlreadyResident => err == PhysicalResidencyDenial::FrameAlreadyResident,
            Self::Pressure => matches!(err, PhysicalResidencyDenial::Pressure(_)),
        }
    }
}
