use super::ReplayEvent;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ReplayEvent {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            cursor: _,
            kind: _,
            branch_id: _,
            snapshot_id: _,
            node: _,
            execution_record_id: _,
            semantic_segment_id: _,
            lineage_artifact_id: _,
            reuse_origin: _,
            persistent_correspondence_kind: _,
            composition_region_count: _,
            detail,
        } = self;
        detail.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for super::ReplayEventDetail {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::TaskOutcome(_) => Ok(Charge::ZERO),
            Self::Message(message) => message.retained_heap_charge(work),
        }
    }
}

impl RetainedStorageMeasurement for super::ReplayCursor {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(_) = self;
        Ok(Charge::ZERO)
    }
}
