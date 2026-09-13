use super::{FlowPayload, RetainedFlow};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for RetainedFlow {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            payload,
            event_epochs,
            observation,
        } = self;
        payload
            .retained_heap_charge(work)?
            .checked_add(event_epochs.retained_heap_charge(work)?)?
            .checked_add(observation.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for FlowPayload {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            profile: _,
            change,
            invalidation,
            planning,
            precompute,
            apply,
            cause_samples,
            rollback,
            explanation,
        } = self;
        change
            .retained_heap_charge(work)?
            .checked_add(invalidation.retained_heap_charge(work)?)?
            .checked_add(planning.retained_heap_charge(work)?)?
            .checked_add(precompute.retained_heap_charge(work)?)?
            .checked_add(apply.retained_heap_charge(work)?)?
            .checked_add(cause_samples.retained_heap_charge(work)?)?
            .checked_add(rollback.retained_heap_charge(work)?)?
            .checked_add(explanation.retained_heap_charge(work)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::retained_storage::arc_allocation_charge;
    use crate::diagnostics::epochs::EventEpochSummary;
    use crate::logic::transaction::ObservationBoundarySummary;
    #[test]
    fn retained_flow_charges_actual_arc_headers_without_cloning_payload_capacity() {
        let flow = super::super::tests::recorded_flow();
        let logical_heap = flow.retained_heap_charge(&mut Work::new(100000)).unwrap();
        let mut expected = logical_heap
            .checked_add(arc_allocation_charge::<FlowPayload>().unwrap())
            .unwrap()
            .checked_add(arc_allocation_charge::<Vec<EventEpochSummary>>().unwrap())
            .unwrap();
        if flow.observation.is_some() {
            expected = expected
                .checked_add(arc_allocation_charge::<ObservationBoundarySummary>().unwrap())
                .unwrap();
        }
        let retained: RetainedFlow = flow.into();
        assert_eq!(
            retained
                .retained_heap_charge(&mut Work::new(100000))
                .unwrap(),
            expected
        );
    }
}
