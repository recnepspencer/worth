//! Native marking spends its own installed ceiling, apart from the
//! publication's bookkeeping, so reader fan-out can degrade delivery but can
//! never refuse a legal writer.

use worth_relational::facade::mvcc::{
    CompanionPreflightBudget, CompanionPreflightStop, PublicationCompanionPreflight,
};

use super::super::admission::{IndexAdmission, RetainedIndexAdmission};

/// Matching, propagation and the hints they select are charged here. Its
/// preparation bytes are transferred to the publication only when the marked
/// state is kept; a degraded delivery drops them with the discarded clone.
pub(super) struct MarkingMeter<'a, 'b> {
    context: &'a PublicationCompanionPreflight<'b>,
    budget: CompanionPreflightBudget,
    work: u64,
    bytes: u64,
    index_bytes: u64,
}

impl<'a, 'b> MarkingMeter<'a, 'b> {
    pub(super) fn new(
        context: &'a PublicationCompanionPreflight<'b>,
        budget: CompanionPreflightBudget,
    ) -> Self {
        Self {
            context,
            budget,
            work: 0,
            bytes: 0,
            index_bytes: 0,
        }
    }

    /// Probing an admitted commit's own touch keys is linear in that commit;
    /// it observes interruption but spends no marking work.
    pub(super) fn checkpoint(&self) -> Result<(), CompanionPreflightStop> {
        self.context.checkpoint()
    }

    pub(super) const fn charged_bytes(&self) -> u64 {
        self.bytes
    }

    pub(super) const fn charged_index_bytes(&self) -> u64 {
        self.index_bytes
    }
}

impl RetainedIndexAdmission for MarkingMeter<'_, '_> {
    fn record_index_bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop> {
        self.index_bytes = self
            .index_bytes
            .checked_add(bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        Ok(())
    }
}

impl IndexAdmission for MarkingMeter<'_, '_> {
    fn work(&mut self, visits: u64) -> Result<(), CompanionPreflightStop> {
        self.context.checkpoint()?;
        let required = self
            .work
            .checked_add(visits)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        if required > self.budget.maximum_work_visits {
            return Err(CompanionPreflightStop::WorkExhausted {
                required,
                maximum: self.budget.maximum_work_visits,
            });
        }
        self.work = required;
        Ok(())
    }

    fn bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop> {
        self.context.checkpoint()?;
        let required = self
            .bytes
            .checked_add(bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        if required > self.budget.maximum_preparation_bytes {
            return Err(CompanionPreflightStop::PreparationMemoryExhausted {
                required,
                maximum: self.budget.maximum_preparation_bytes,
            });
        }
        self.bytes = required;
        Ok(())
    }

    /// Navigation along shared mark indexes observes interruption but spends
    /// no marking work: the ceiling bounds matched postings and their closure.
    fn navigation(&mut self, _units: u64) -> Result<(), CompanionPreflightStop> {
        self.context.checkpoint()
    }
}

/// Marking exhaustion is answered by a counted delivery discontinuity. Every
/// other stop belongs to the publication itself and still defers the writer.
pub(super) const fn degrades_delivery(stop: &CompanionPreflightStop) -> bool {
    match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow
        | CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow => true,
        CompanionPreflightStop::TopologyPending
        | CompanionPreflightStop::SelectedSourceMismatch
        | CompanionPreflightStop::SelectedPositionUnavailable { .. }
        | CompanionPreflightStop::ForeignCell
        | CompanionPreflightStop::RegistrationChanged
        | CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }
        | CompanionPreflightStop::Interrupted(_) => false,
    }
}

/// Hint custody whose preparation bytes were already admitted with the marking
/// that selected it. Drawing past that allowance is a counter defect.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) struct PrepaidAdmission
{
    work: u64,
    bytes: u64,
}

impl PrepaidAdmission {
    pub(super) const fn new(work: u64, bytes: u64) -> Self {
        Self { work, bytes }
    }
}

impl IndexAdmission for PrepaidAdmission {
    fn work(&mut self, visits: u64) -> Result<(), CompanionPreflightStop> {
        // A draw past the prepaid allowance is an accounting defect, reported
        // through the existing counter-defect stop.
        debug_assert!(visits <= self.work, "prepaid hint work underflow");
        self.work = self
            .work
            .checked_sub(visits)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        Ok(())
    }

    fn bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop> {
        debug_assert!(bytes <= self.bytes, "prepaid hint bytes underflow");
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        Ok(())
    }

    /// Hint construction edits no shared index; nothing was prepaid for it.
    fn navigation(&mut self, _units: u64) -> Result<(), CompanionPreflightStop> {
        Ok(())
    }
}
