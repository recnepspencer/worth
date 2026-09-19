use super::{NonEmptyCanonicalAspectChangeSet, ProducedAspectChange, ProducedAspectDelta};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ProducedAspectChange {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            aspect: _,
            previous_version: _,
            committed_version: _,
            changed_scopes,
        } = self;
        changed_scopes.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for ProducedAspectDelta {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            producer: _,
            output_commit_ordinal: _,
            committed_output_version: _,
            changes,
            scope_precision: _,
        } = self;
        let NonEmptyCanonicalAspectChangeSet(changes) = changes;
        changes.retained_heap_charge(work)
    }
}
