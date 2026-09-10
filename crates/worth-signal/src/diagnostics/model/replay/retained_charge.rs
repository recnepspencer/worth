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
            Self::BranchMergeSummary {
                message,
                strategy_witness,
                compatibility_witness,
                scoped_merge_proof,
            } => message
                .retained_heap_charge(work)?
                .checked_add(strategy_witness.retained_heap_charge(work)?)?
                .checked_add(compatibility_witness.retained_heap_charge(work)?)?
                .checked_add(scoped_merge_proof.retained_heap_charge(work)?),
        }
    }
}

#[cfg(test)]
mod tests;

impl RetainedStorageMeasurement for super::ReplayCursor {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(_) = self;
        Ok(Charge::ZERO)
    }
}
