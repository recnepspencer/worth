use super::*;
use crate::data::graph::storage::NodeEvaluationMutation;

#[test]
fn retained_draft_admits_warm_and_cold_byte_copies_before_the_edit_callback() {
    let payload = "large payload".repeat(4096);
    for cold_payload in [false, true] {
        let mut arena = prepared_arena();
        if cold_payload {
            arena.cold[0] = Some(Box::new(NodeColdData {
                causality: Some(crate::data::trace::CausalityMetadata {
                    kind: payload.clone(),
                    fields: Default::default(),
                }),
                ..Default::default()
            }));
            arena
                .cold
                .prepare_retained_charge(&mut Work::new(1000))
                .unwrap();
        } else {
            arena.warm[0].dirty_partition_scope_payload.push((
                crate::data::aspect::Aspect::new(0),
                crate::data::output::PartitionSubscription::whole_partition(payload.as_str()),
            ));
            arena
                .warm
                .prepare_retained_charge(&mut Work::new(1000))
                .unwrap();
        }
        let sibling = arena.clone();
        let maximum = representation_charge(&arena).checked_mul(2).unwrap();
        let maximum_visits = payload.len() - 1;
        let denied = arena.prepare_retained_node_edits(
            &test_ledger(),
            &[0],
            maximum,
            &mut Work::new(maximum_visits),
            |_| panic!("copy work must be admitted before the edit"),
        );
        assert!(matches!(denied, Err(RetainedNodeEditDenial::Accounting(
            crate::data::retained_storage::RetainedStoragePreparationDenial::WorkExhausted { maximum_visits: actual }
        )) if actual == maximum_visits));
        assert!(arena.hot.shares_storage_with(&sibling.hot));
        assert!(arena.warm.shares_storage_with(&sibling.warm));
        assert!(arena.cold.shares_storage_with(&sibling.cold));

        let mut work = Work::new(200_000);
        let RetainedNodeEditPreparation::Ready(prepared) = arena
            .prepare_retained_node_edits(&test_ledger(), &[0], maximum, &mut work, |payloads| {
                let payload = &mut payloads[0];
                NodeEvaluationMutation::draft(
                    &mut payload.hot,
                    &mut payload.warm,
                    &mut payload.cold,
                )
                .set_state(NodeState::Clean);
            })
            .unwrap()
        else {
            panic!("admitted payload copy must prepare");
        };
        assert!(work.visits() >= payload.len());
        assert!(matches!(
            prepared.install(&mut arena),
            RetainedNodeEditOutcome::Installed { .. }
        ));
        assert_eq!(arena.hot[0].as_ref().unwrap().state, NodeState::Clean);
        assert_eq!(arena.warm, sibling.warm);
        assert_eq!(arena.cold, sibling.cold);
    }
}
