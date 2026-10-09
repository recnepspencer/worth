use super::super::frontier::BlobStreamingContentFrontier;
use super::super::receipt_construction::BlobStreamingCounterBackedPerformanceReceipt;
use crate::{
    AdmittedBlobChunkSequence, BlobStreamingIngestCounterSnapshot, BlobStreamingResumeAdmission,
    BlobStreamingResumePosture,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobStreamingIngest {
    pub(crate) sequence: AdmittedBlobChunkSequence,
    pub(crate) frontier: BlobStreamingContentFrontier,
    pub(crate) resumability: BlobStreamingResumePosture,
    pub(crate) counters: BlobStreamingIngestCounterSnapshot,
    pub(crate) performance: BlobStreamingCounterBackedPerformanceReceipt,
}

impl BlobStreamingIngest {
    pub(crate) const fn from_bounded_parts(
        sequence: AdmittedBlobChunkSequence,
        frontier: BlobStreamingContentFrontier,
        resumability: BlobStreamingResumePosture,
        counters: BlobStreamingIngestCounterSnapshot,
        performance: BlobStreamingCounterBackedPerformanceReceipt,
    ) -> Self {
        Self {
            sequence,
            frontier,
            resumability,
            counters,
            performance,
        }
    }

    pub const fn sequence(&self) -> &AdmittedBlobChunkSequence {
        &self.sequence
    }

    pub const fn frontier(&self) -> &BlobStreamingContentFrontier {
        &self.frontier
    }

    pub const fn resumability(&self) -> &BlobStreamingResumePosture {
        &self.resumability
    }

    pub const fn counters(&self) -> BlobStreamingIngestCounterSnapshot {
        self.counters
    }

    pub const fn counter_backed_performance_receipt(
        &self,
    ) -> &BlobStreamingCounterBackedPerformanceReceipt {
        &self.performance
    }

    pub(crate) fn bind_resume_admission(mut self, admission: BlobStreamingResumeAdmission) -> Self {
        self.resumability = self
            .resumability
            .with_resume_session(admission.session_digest());
        self
    }
}
