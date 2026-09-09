use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
impl<T: RetainedStorageMeasurement> RetainedStorageMeasurement for std::collections::BTreeSet<T> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let mut charge = super::btree_structure_charge::<T, ()>(self.len())?;
        for item in self {
            charge = charge.checked_add(item.retained_heap_charge(work)?)?;
        }
        Ok(charge)
    }
}
impl<T: Clone + Ord + RetainedStorageMeasurement> RetainedStorageMeasurement for im::OrdSet<T> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let mut charge = super::ordered_index_charge::<T, ()>(self.len())?;
        for item in self {
            charge = charge.checked_add(item.retained_heap_charge(work)?)?;
        }
        Ok(charge)
    }
}
impl<K: Clone + Ord + RetainedStorageMeasurement, V: Clone + RetainedStorageMeasurement>
    RetainedStorageMeasurement for im::OrdMap<K, V>
{
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let mut charge = super::ordered_index_charge::<K, V>(self.len())?;
        for (key, value) in self {
            charge = charge
                .checked_add(key.retained_heap_charge(work)?)?
                .checked_add(value.retained_heap_charge(work)?)?;
        }
        Ok(charge)
    }
}
