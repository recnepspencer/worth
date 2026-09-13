use worth_store_wal::{WalLsnRange, WalSegmentArtifactIdentity, WalSegmentInspection};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalWalSegmentCandidate {
    inspection: WalSegmentInspection,
    interrupted_tail: Option<PhysicalWalInterruptionFacts>,
    frame_facts: Box<[PhysicalWalFrameFacts]>,
    selected_frame_start: usize,
    selected_range: Option<WalLsnRange>,
    selected_bytes: Option<u64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalWalFrameFacts {
    lsn_range: WalLsnRange,
    encoded_bytes: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalWalInterruptionFacts {
    valid_prefix_bytes: u64,
    observed_bytes: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedPhysicalWalTail {
    segments: Vec<PhysicalWalSegmentCandidate>,
    checkpoint_covered: Vec<super::CheckpointCoveredWalArtifact>,
    frame_count: u64,
    byte_count: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedPhysicalWalTailDenial {
    DuplicateArtifact,
    GenerationMismatch,
    SegmentGap,
    LsnGap,
    CheckpointFrontierMismatch,
    InterruptedMiddleSegment,
    CounterOverflow,
}
impl PhysicalWalSegmentCandidate {
    pub fn from_frame_facts(
        inspection: WalSegmentInspection,
        interrupted_tail: Option<PhysicalWalInterruptionFacts>,
        frame_facts: Vec<PhysicalWalFrameFacts>,
    ) -> Option<Self> {
        if frame_facts.len() as u64 != inspection.frame_count()
            || frame_facts.first()?.lsn_range().start() != inspection.lsn_range().start()
            || frame_facts.last()?.lsn_range().end_exclusive()
                != inspection.lsn_range().end_exclusive()
            || frame_facts
                .windows(2)
                .any(|pair| pair[0].lsn_range().end_exclusive() != pair[1].lsn_range().start())
            || frame_facts.iter().try_fold(0_u64, |total, frame| {
                total.checked_add(frame.encoded_bytes())
            })? != inspection.byte_count()
        {
            return None;
        }
        Some(Self {
            inspection,
            interrupted_tail,
            frame_facts: frame_facts.into_boxed_slice(),
            selected_frame_start: 0,
            selected_range: None,
            selected_bytes: None,
        })
    }

    pub const fn identity(&self) -> WalSegmentArtifactIdentity {
        self.inspection.identity()
    }

    pub const fn inspection(&self) -> WalSegmentInspection {
        self.inspection
    }

    pub const fn interrupted_tail(&self) -> Option<PhysicalWalInterruptionFacts> {
        self.interrupted_tail
    }

    pub fn frame_facts(&self) -> &[PhysicalWalFrameFacts] {
        &self.frame_facts[self.selected_frame_start..]
    }

    fn selected_range(&self) -> WalLsnRange {
        self.selected_range
            .unwrap_or_else(|| self.inspection.lsn_range())
    }

    fn selected_frame_count(&self) -> u64 {
        self.selected_range
            .map_or(self.inspection.frame_count(), |_| {
                self.frame_facts().len() as u64
            })
    }

    fn selected_byte_count(&self) -> u64 {
        self.selected_bytes
            .unwrap_or_else(|| self.inspection.byte_count())
    }

    fn trim_before(mut self, frontier: u64) -> Result<Option<Self>, SelectedPhysicalWalTailDenial> {
        let range = self.inspection.lsn_range();
        if range.end_exclusive().get() <= frontier {
            return Ok(None);
        }
        if range.start().get() >= frontier {
            return Ok(Some(self));
        }
        let first = self
            .frame_facts()
            .iter()
            .position(|frame| frame.lsn_range().start().get() >= frontier)
            .ok_or(SelectedPhysicalWalTailDenial::CheckpointFrontierMismatch)?;
        if self.frame_facts()[first].lsn_range().start().get() != frontier {
            return Err(SelectedPhysicalWalTailDenial::CheckpointFrontierMismatch);
        }
        self.selected_frame_start = self
            .selected_frame_start
            .checked_add(first)
            .ok_or(SelectedPhysicalWalTailDenial::CounterOverflow)?;
        let start = self.frame_facts().first().unwrap().lsn_range().start();
        let end = self
            .frame_facts()
            .last()
            .unwrap()
            .lsn_range()
            .end_exclusive();
        self.selected_range = Some(
            WalLsnRange::new(start, end)
                .map_err(|_| SelectedPhysicalWalTailDenial::CheckpointFrontierMismatch)?,
        );
        self.selected_bytes = Some(
            self.frame_facts()
                .iter()
                .try_fold(0_u64, |bytes, frame| {
                    bytes.checked_add(frame.encoded_bytes())
                })
                .ok_or(SelectedPhysicalWalTailDenial::CounterOverflow)?,
        );
        Ok(Some(self))
    }
}

pub fn admit_physical_wal_tail(
    checkpoint_frontier: u64,
    mut candidates: Vec<PhysicalWalSegmentCandidate>,
) -> Result<SelectedPhysicalWalTail, SelectedPhysicalWalTailDenial> {
    candidates.sort_unstable_by_key(|candidate| candidate.identity());
    if candidates
        .windows(2)
        .any(|pair| pair[0].identity() == pair[1].identity())
    {
        return Err(SelectedPhysicalWalTailDenial::DuplicateArtifact);
    }
    let mut checkpoint_covered = Vec::new();
    let mut retained = Vec::new();
    for candidate in candidates {
        if candidate.inspection().lsn_range().end_exclusive().get() <= checkpoint_frontier {
            checkpoint_covered.push(super::CheckpointCoveredWalArtifact::from_candidate(
                candidate,
            ));
            continue;
        }
        if let Some(candidate) = candidate.trim_before(checkpoint_frontier)? {
            retained.push(candidate);
        }
    }
    candidates = retained;
    let mut frame_count = 0_u64;
    let mut byte_count = 0_u64;
    for (index, candidate) in candidates.iter().enumerate() {
        crate::wal_prefix::classify_terminal_interruption(
            index,
            candidates.len(),
            candidate.interrupted_tail(),
        )
        .map_err(map_prefix_denial)?;
        if let Some(previous) = index.checked_sub(1).map(|prior| &candidates[prior]) {
            if previous.identity() == candidate.identity() {
                return Err(SelectedPhysicalWalTailDenial::DuplicateArtifact);
            }
            if previous.identity().generation() != candidate.identity().generation() {
                return Err(SelectedPhysicalWalTailDenial::GenerationMismatch);
            }
            if previous.identity().segment().get().checked_add(1)
                != Some(candidate.identity().segment().get())
            {
                return Err(SelectedPhysicalWalTailDenial::SegmentGap);
            }
        }
        frame_count = frame_count
            .checked_add(candidate.selected_frame_count())
            .ok_or(SelectedPhysicalWalTailDenial::CounterOverflow)?;
        byte_count = byte_count
            .checked_add(candidate.selected_byte_count())
            .ok_or(SelectedPhysicalWalTailDenial::CounterOverflow)?;
    }
    crate::wal_prefix::require_contiguous_prefix(
        checkpoint_frontier,
        candidates
            .iter()
            .map(PhysicalWalSegmentCandidate::selected_range),
    )
    .map_err(map_prefix_denial)?;
    let facts = crate::wal_prefix::WalValidPrefixFacts {
        frame_count,
        byte_count,
    };
    Ok(SelectedPhysicalWalTail {
        segments: candidates,
        checkpoint_covered,
        frame_count: facts.frame_count,
        byte_count: facts.byte_count,
    })
}

fn map_prefix_denial(
    denial: crate::wal_prefix::WalPrefixAdmissionDenial,
) -> SelectedPhysicalWalTailDenial {
    match denial {
        crate::wal_prefix::WalPrefixAdmissionDenial::FrontierMismatch => {
            SelectedPhysicalWalTailDenial::CheckpointFrontierMismatch
        }
        crate::wal_prefix::WalPrefixAdmissionDenial::Gap => SelectedPhysicalWalTailDenial::LsnGap,
        crate::wal_prefix::WalPrefixAdmissionDenial::InterruptedMiddle => {
            SelectedPhysicalWalTailDenial::InterruptedMiddleSegment
        }
    }
}

impl SelectedPhysicalWalTail {
    pub fn segments(&self) -> &[PhysicalWalSegmentCandidate] {
        &self.segments
    }

    pub fn checkpoint_covered(&self) -> &[super::CheckpointCoveredWalArtifact] {
        &self.checkpoint_covered
    }

    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    pub fn frame_facts(&self) -> impl Iterator<Item = &PhysicalWalFrameFacts> {
        self.segments
            .iter()
            .flat_map(|segment| segment.frame_facts())
    }
}

impl PhysicalWalFrameFacts {
    pub fn new(lsn_range: WalLsnRange, encoded_bytes: u64) -> Option<Self> {
        (encoded_bytes != 0).then_some(Self {
            lsn_range,
            encoded_bytes,
        })
    }

    pub const fn lsn_range(self) -> WalLsnRange {
        self.lsn_range
    }

    pub const fn encoded_bytes(self) -> u64 {
        self.encoded_bytes
    }
}

impl PhysicalWalInterruptionFacts {
    pub fn new(valid_prefix_bytes: u64, observed_bytes: u64) -> Option<Self> {
        (valid_prefix_bytes != 0 && observed_bytes > valid_prefix_bytes).then_some(Self {
            valid_prefix_bytes,
            observed_bytes,
        })
    }

    pub const fn valid_prefix_bytes(self) -> u64 {
        self.valid_prefix_bytes
    }

    pub const fn observed_bytes(self) -> u64 {
        self.observed_bytes
    }
}

#[cfg(test)]
mod tests;
