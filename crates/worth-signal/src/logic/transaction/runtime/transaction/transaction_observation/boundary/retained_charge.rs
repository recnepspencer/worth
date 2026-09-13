use super::{CommittedObservationEventSummary, ObservationBoundarySummary};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for CommittedObservationEventSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            observer_id: _,
            handle_id: _,
            policy: _,
            observed_nodes,
            matched_nodes,
            touched: _,
            recomputed: _,
            meaningful_change: _,
            trigger_matched: _,
            outcome: _,
        } = self;
        observed_nodes
            .retained_heap_charge(work)?
            .checked_add(matched_nodes.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for ObservationBoundarySummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            classified_event_count: _,
            trigger_matched_event_count: _,
            delivered_event_count: _,
            rollback_suppressed_event_count: _,
            branch_local_suppressed_event_count: _,
            boundary_events,
        } = self;
        boundary_events.retained_heap_charge(work)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::retained_storage::btree_structure_charge;
    use crate::facade::{SignalGraph, SignalRuntime};
    use crate::logic::transaction::{
        ObservationListener, ObservationNotice, ObservationPolicy, ObservationReadContext,
        ObservedNodeSet,
    };
    struct Listener;
    impl ObservationListener<(), (), (), (), ()> for Listener {
        fn on_observation(
            &self,
            _: ObservationReadContext<'_, (), (), (), (), ()>,
            _: &ObservationNotice<'_>,
        ) {
        }
    }
    #[test]
    fn observation_summary_charges_both_node_sets_and_event_capacity() {
        let mut graph = SignalGraph::new();
        let nodes = [graph.node().build(), graph.node().build()];
        let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
        let handle = runtime.observe_nodes(ObservationPolicy::touched(), nodes, Box::new(Listener));
        // Retained-summary fixture uses owner-issued handle identities.
        let mut boundary_events = Vec::with_capacity(16);
        boundary_events.push(CommittedObservationEventSummary {
            observer_id: handle.observer_id(),
            handle_id: handle.handle_id(),
            policy: ObservationPolicy::touched(),
            observed_nodes: ObservedNodeSet::from_nodes(nodes),
            matched_nodes: ObservedNodeSet::from_nodes([nodes[0]]),
            touched: true,
            recomputed: false,
            meaningful_change: false,
            trigger_matched: true,
            outcome: super::super::ObservationBoundaryOutcome::Delivered,
        });
        let expected =
            boundary_events.capacity() * std::mem::size_of::<CommittedObservationEventSummary>();
        let summary = ObservationBoundarySummary {
            boundary_events,
            ..Default::default()
        };
        let expected = Charge::capacity::<u8>(expected)
            .unwrap()
            .checked_add(btree_structure_charge::<crate::data::handle::NodeId, ()>(2).unwrap())
            .unwrap()
            .checked_add(btree_structure_charge::<crate::data::handle::NodeId, ()>(1).unwrap())
            .unwrap();
        assert_eq!(
            summary.retained_heap_charge(&mut Work::new(1000)).unwrap(),
            expected
        );
    }
}
