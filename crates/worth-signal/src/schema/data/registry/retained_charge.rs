use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::SignalSchemaRegistry;

impl RetainedStorageMeasurement for SignalSchemaRegistry {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            registrations,
            index_by_id,
            index_by_name,
            registry_digest,
        } = self;
        Ok(Charge::ZERO
            .checked_add(registrations.retained_heap_charge(work)?)?
            .checked_add(index_by_id.retained_heap_charge(work)?)?
            .checked_add(index_by_name.retained_heap_charge(work)?)?
            .checked_add(registry_digest.retained_heap_charge(work)?)?)
    }
}
