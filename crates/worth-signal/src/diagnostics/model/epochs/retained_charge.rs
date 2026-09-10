use super::{EventEpochSummary, EventSubscriberOutcome};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for EventSubscriberOutcome {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            subscriber_name,
            outcome: _,
            requires_data_ids,
            provides_data_ids,
            staged_data_ids,
        } = self;
        subscriber_name
            .retained_heap_charge(work)?
            .checked_add(requires_data_ids.retained_heap_charge(work)?)?
            .checked_add(provides_data_ids.retained_heap_charge(work)?)?
            .checked_add(staged_data_ids.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for EventEpochSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            ordinal: _,
            barrier: _,
            emitted_event_count: _,
            subscriber_count: _,
            committed_subscriber_count: _,
            failed_subscriber_position: _,
            subscriber_outcomes,
            outcome: _,
            failure_subscriber,
            message,
        } = self;
        subscriber_outcomes
            .retained_heap_charge(work)?
            .checked_add(failure_subscriber.retained_heap_charge(work)?)?
            .checked_add(message.retained_heap_charge(work)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::epochs::{EventEpochOutcome, EventSubscriberOutcomeKind};
    #[test]
    fn epoch_charge_covers_spare_subscriber_vectors_and_all_owned_text() {
        let mut strings = Vec::new();
        let mut string_bytes = 0;
        for capacity in [512, 1024, 2048, 4096, 8192, 16384] {
            let value = String::with_capacity(capacity);
            string_bytes += value.capacity();
            strings.push(value);
        }
        let message = strings.pop().unwrap();
        let failure_subscriber = strings.pop().unwrap();
        let subscriber_name = strings.pop().unwrap();
        let mut requires_data_ids = Vec::with_capacity(8);
        let mut provides_data_ids = Vec::with_capacity(16);
        let mut staged_data_ids = Vec::with_capacity(32);
        requires_data_ids.push(strings.pop().unwrap());
        provides_data_ids.push(strings.pop().unwrap());
        staged_data_ids.push(strings.pop().unwrap());
        let vector_bytes = (requires_data_ids.capacity()
            + provides_data_ids.capacity()
            + staged_data_ids.capacity())
            * std::mem::size_of::<String>();
        let mut subscriber_outcomes = Vec::with_capacity(4);
        subscriber_outcomes.push(EventSubscriberOutcome {
            subscriber_name,
            outcome: EventSubscriberOutcomeKind::Failed,
            requires_data_ids,
            provides_data_ids,
            staged_data_ids,
        });
        let expected = string_bytes
            + vector_bytes
            + subscriber_outcomes.capacity() * std::mem::size_of::<EventSubscriberOutcome>();
        // Explicit retained representation, not a dispatched subscriber event.
        let epoch = EventEpochSummary {
            ordinal: 0,
            barrier: crate::data::checkpoint::CheckpointBarrier::PerOperation,
            emitted_event_count: 1,
            subscriber_count: 1,
            committed_subscriber_count: 0,
            failed_subscriber_position: Some(0),
            subscriber_outcomes,
            outcome: EventEpochOutcome::Failed,
            failure_subscriber: Some(failure_subscriber),
            message: Some(message),
        };
        assert_eq!(
            epoch
                .retained_heap_charge(&mut Work::new(100))
                .unwrap()
                .bytes(),
            expected as u64
        );
    }
}
