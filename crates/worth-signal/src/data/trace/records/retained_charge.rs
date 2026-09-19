use super::{ExecutionTraceStamp, RetainedDiagnosticArtifact};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ExecutionTraceStamp {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            execution_record_id: _,
            semantic_segment_id: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for RetainedDiagnosticArtifact {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            changed_regions,
            labels,
            keyed_family,
            keyed_key,
            reuse_certification,
            reuse_boundary_context,
        } = self;
        changed_regions
            .retained_heap_charge(work)?
            .checked_add(labels.retained_heap_charge(work)?)?
            .checked_add(keyed_family.retained_heap_charge(work)?)?
            .checked_add(keyed_key.retained_heap_charge(work)?)?
            .checked_add(reuse_certification.retained_heap_charge(work)?)?
            .checked_add(reuse_boundary_context.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for super::HistoricalArtifactRecord {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            runtime,
            retained,
            causality,
        } = self;
        runtime
            .retained_heap_charge(work)?
            .checked_add(retained.retained_heap_charge(work)?)?
            .checked_add(causality.retained_heap_charge(work)?)
    }
}
