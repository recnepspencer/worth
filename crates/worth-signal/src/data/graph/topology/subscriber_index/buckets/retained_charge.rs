use super::{
    DetailScopeKey, IndexedSubscriptionMembership, IndexedSubscriptionScope, ProducerAspectKey,
    ReverseSubscriptionIndex, SubscriberScopeBuckets,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl RetainedStorageForkPreparation for ReverseSubscriptionIndex {
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        work.visit()?;
        let Self { storage, valid: _ } = self;
        storage.prepare_fork_charge(work)
    }
}
impl RetainedStorageMeasurement for ProducerAspectKey {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            producer: _,
            aspect: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for DetailScopeKey {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            partition: _,
            detail: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for IndexedSubscriptionScope {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Unscoped | Self::WholePartition(_) | Self::Detail(_, _) => Ok(Charge::ZERO),
        }
    }
}
impl RetainedStorageMeasurement for IndexedSubscriptionMembership {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { scope, key: _ } = self;
        Ok(Charge::ZERO.checked_add(scope.retained_heap_charge(work)?)?)
    }
}
impl RetainedStorageMeasurement for SubscriberScopeBuckets {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            all,
            unscoped,
            whole_partitions,
            exact_details,
            partition_scoped,
        } = self;
        Ok(Charge::ZERO
            .checked_add(all.retained_heap_charge(work)?)?
            .checked_add(unscoped.retained_heap_charge(work)?)?
            .checked_add(whole_partitions.retained_heap_charge(work)?)?
            .checked_add(exact_details.retained_heap_charge(work)?)?
            .checked_add(partition_scoped.retained_heap_charge(work)?)?)
    }
}
impl RetainedStorageMeasurement for ReverseSubscriptionIndex {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { storage, valid: _ } = self;
        Ok(Charge::ZERO.checked_add(storage.retained_heap_charge(work)?)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{aspect::Aspect, handle::NodeId};

    #[test]
    fn removed_consumer_keeps_shared_index_allocation_charged() {
        let producer = NodeId::new(0, 0);
        let consumer = NodeId::new(1, 0);
        let mut samples = Vec::new();
        for requested_capacity in [1, 8_192] {
            let mut source = ReverseSubscriptionIndex::default();
            let mut memberships = Vec::with_capacity(requested_capacity);
            memberships.push(
                IndexedSubscriptionMembership::from_edge(producer, Aspect::new(0), None).unwrap(),
            );
            let capacity_bytes =
                memberships.capacity() * std::mem::size_of::<IndexedSubscriptionMembership>();
            source.replace_consumer(consumer, memberships);
            let mut retained = source.fork_persistent();
            retained.replace_consumer(consumer, Vec::new());
            drop(source);
            assert!(retained
                .query_whole_aspect(
                    producer,
                    Aspect::new(0),
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap()
                .candidates
                .is_empty());
            let mut work = Preparation::new(1_000);
            let charge = retained.retained_heap_charge(&mut work).unwrap();
            assert_eq!(
                retained
                    .retained_heap_charge(&mut Preparation::new(work.visits()))
                    .unwrap(),
                charge
            );
            assert!(matches!(
                retained.retained_heap_charge(&mut Preparation::new(work.visits() - 1)),
                Err(Denial::WorkExhausted { .. })
            ));
            samples.push((capacity_bytes as u64, charge.bytes()));
        }
        // Identical visible membership and overlays; only the now-hidden base
        // vector's spare capacity differs between the two lawful histories.
        assert_eq!(samples[1].1 - samples[0].1, samples[1].0 - samples[0].0);
    }
}
