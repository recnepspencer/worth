use super::{Arc, CandidateFrameCounterCells, PhysicalResidencyCounters, PhysicalResidencyPool};

pub struct FramePortCounterObserver {
    pub(super) pool: PhysicalResidencyPool,
    pub(super) candidate_counters: Arc<CandidateFrameCounterCells>,
}

impl FramePortCounterObserver {
    pub fn snapshot(&self) -> FramePortCounterSnapshot {
        FramePortCounterSnapshot {
            residency: self.pool.counters(),
            candidate_submissions: self.candidate_counters.submissions(),
            declared_candidate_frames: self.candidate_counters.declared_frames(),
            declared_candidate_bytes: self.candidate_counters.declared_bytes(),
            candidate_frames: self.candidate_counters.retained_frames(),
            candidate_bytes: self.candidate_counters.retained_bytes(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramePortCounterSnapshot {
    residency: PhysicalResidencyCounters,
    candidate_submissions: u64,
    declared_candidate_frames: u64,
    declared_candidate_bytes: u64,
    candidate_frames: u64,
    candidate_bytes: u64,
}

impl FramePortCounterSnapshot {
    pub const fn loads(self) -> u64 {
        self.residency.source_loads()
    }
    pub const fn candidate_submissions(self) -> u64 {
        self.candidate_submissions
    }
    pub const fn declared_candidate_frames(self) -> u64 {
        self.declared_candidate_frames
    }
    pub const fn declared_candidate_bytes(self) -> u64 {
        self.declared_candidate_bytes
    }
    pub const fn candidate_frames(self) -> u64 {
        self.candidate_frames
    }
    pub const fn candidate_bytes(self) -> u64 {
        self.candidate_bytes
    }
    pub const fn wrapper_frames(self) -> u64 {
        0
    }
    pub const fn peak_retained_candidate_frames(self) -> u64 {
        self.residency.peak_candidate_frames() as u64
    }
    pub const fn residency_hits(self) -> u64 {
        self.residency.hits()
    }
    pub const fn residency_faults(self) -> u64 {
        self.residency.faults()
    }
    pub const fn writebacks(self) -> u64 {
        self.residency.writebacks()
    }
    pub const fn candidate_publications(self) -> u64 {
        self.residency.candidate_publications()
    }
    pub const fn evictions(self) -> u64 {
        self.residency.evictions()
    }
}
