use super::SignalMergeStrategyWitness;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for SignalMergeStrategyWitness {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            merge_strategy,
            invalidation_strategy,
            delivery_strategy,
            merge_strategy_digest,
            invalidation_strategy_digest,
            delivery_strategy_digest,
            witness_digest,
        } = self;
        merge_strategy
            .retained_heap_charge(work)?
            .checked_add(invalidation_strategy.retained_heap_charge(work)?)?
            .checked_add(delivery_strategy.retained_heap_charge(work)?)?
            .checked_add(merge_strategy_digest.retained_heap_charge(work)?)?
            .checked_add(invalidation_strategy_digest.retained_heap_charge(work)?)?
            .checked_add(delivery_strategy_digest.retained_heap_charge(work)?)?
            .checked_add(witness_digest.retained_heap_charge(work)?)
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
            .as_strategy_witness()
            .unwrap()
            .clone();
        let original = value.clone();
        let before = value.retained_heap_charge(&mut Work::new(10000)).unwrap();
        let mut delta = 0;
        for field in [
            &mut value.merge_strategy_digest,
            &mut value.invalidation_strategy_digest,
            &mut value.delivery_strategy_digest,
            &mut value.witness_digest,
        ] {
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
