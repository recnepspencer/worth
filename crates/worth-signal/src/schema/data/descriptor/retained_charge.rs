use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::{SignalSchemaDescriptor, SignalSchemaId, SignalSchemaName};

impl RetainedStorageMeasurement for SignalSchemaId {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for SignalSchemaName {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for SignalSchemaDescriptor {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            semantic_name,
            default_contract,
            digest,
            id: _,
            version: _,
        } = self;
        Charge::ZERO
            .checked_add(semantic_name.retained_heap_charge(work)?)?
            .checked_add(default_contract.retained_heap_charge(work)?)?
            .checked_add(digest.retained_heap_charge(work)?)
    }
}
