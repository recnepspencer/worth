//! Candidate validation and frame-boundary trimming preserve supplied storage.

use super::{PhysicalWalFrameFacts, PhysicalWalInterruptionFacts, SelectedPhysicalWalTailDenial};
use worth_store_wal::{WalLsnRange, WalSegmentArtifactIdentity, WalSegmentInspection};

#[derive(Debug, PartialEq, Eq)]
pub struct PhysicalWalSegmentCandidate {
    inspection: WalSegmentInspection,
    interrupted_tail: Option<PhysicalWalInterruptionFacts>,
    frame_facts: Vec<PhysicalWalFrameFacts>,
    selected_frame_start: usize,
    selected_range: Option<WalLsnRange>,
    selected_bytes: Option<u64>,
}
impl PhysicalWalSegmentCandidate {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.frame_facts.capacity())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<PhysicalWalFrameFacts>()).ok()?)
    }

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
            frame_facts,
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

    pub fn frame_fact_capacity(&self) -> usize {
        self.frame_facts.capacity()
    }

    pub fn frame_facts(&self) -> &[PhysicalWalFrameFacts] {
        &self.frame_facts[self.selected_frame_start..]
    }

    pub(super) fn selected_range(&self) -> WalLsnRange {
        self.selected_range
            .unwrap_or_else(|| self.inspection.lsn_range())
    }

    pub(super) fn selected_frame_count(&self) -> u64 {
        self.selected_range
            .map_or(self.inspection.frame_count(), |_| {
                self.frame_facts().len() as u64
            })
    }

    pub(super) fn selected_byte_count(&self) -> u64 {
        self.selected_bytes
            .unwrap_or_else(|| self.inspection.byte_count())
    }

    pub(super) fn trim_before(
        mut self,
        frontier: u64,
    ) -> Result<Option<Self>, SelectedPhysicalWalTailDenial> {
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
