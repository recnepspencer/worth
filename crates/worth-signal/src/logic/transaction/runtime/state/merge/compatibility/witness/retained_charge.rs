use super::SignalMergeCompatibilityWitness;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for SignalMergeCompatibilityWitness {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            schema_version,
            fact_inventory,
            compatibility_digest,
        } = self;
        schema_version
            .retained_heap_charge(work)?
            .checked_add(fact_inventory.retained_heap_charge(work)?)?
            .checked_add(compatibility_digest.retained_heap_charge(work)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::emitted_merge_replay_event;
    fn grow_string(value: &mut String) -> u64 {
        let before = value.capacity();
        value.reserve_exact(1024);
        (value.capacity() - before) as u64
    }
    #[test]
    fn replay_owned_digests_each_contribute_their_capacity() {
        let event = emitted_merge_replay_event();
        let mut value = event
            .detail
            .as_ref()
            .unwrap()
            .as_compatibility_witness()
            .unwrap()
            .clone();
        let original = value.clone();
        let before = value.retained_heap_charge(&mut Work::new(10000)).unwrap();
        let mut delta = 0;
        for field in [&mut value.schema_version, &mut value.compatibility_digest] {
            delta += grow_string(field);
        }
        assert_eq!(
            value
                .retained_heap_charge(&mut Work::new(10000))
                .unwrap()
                .bytes()
                - before.bytes(),
            delta
        );
        assert_eq!(value, original);
    }
}
