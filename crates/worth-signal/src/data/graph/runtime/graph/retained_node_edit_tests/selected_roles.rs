//! Consumer drafts retain cold and immutable warm companion identity.
use super::*;
use crate::data::graph::runtime::graph::retained_node_edit::{SelectedNodeDraft, SelectedNodeRole};

fn edit_roles(payloads: &mut [SelectedNodeDraft], _: &mut Work) -> u32 {
    match &mut payloads[0] {
        SelectedNodeDraft::ProducerFull(payload) => payload.hot.state = NodeState::Clean,
        SelectedNodeDraft::ConsumerOperational(_) => panic!("producer role changed"),
    }
    match &mut payloads[1] {
        SelectedNodeDraft::ConsumerOperational(payload) => {
            payload.hot.state = NodeState::MaybeStale;
            payload.warm.direct_invalidation_generation = 9;
        }
        SelectedNodeDraft::ProducerFull(_) => panic!("consumer role changed"),
    }
    23
}

#[test]
fn consumer_epoch_preserves_cold_payload_and_warm_companion_across_denial_and_install() {
    let mut arena = prepared_arena();
    arena.cold[1] = Some(Box::new(NodeColdData {
        causality: Some(crate::data::trace::CausalityMetadata {
            kind: "retained consumer cold".repeat(4_096),
            fields: Default::default(),
        }),
        ..Default::default()
    }));
    arena
        .cold
        .prepare_retained_charge(&mut Work::new(10_000))
        .unwrap();
    let original = arena.clone();
    let roles = [
        (0, SelectedNodeRole::ProducerFull),
        (1, SelectedNodeRole::ConsumerOperational),
    ];
    let rejected = arena
        .prepare_selected_epoch_nodes(
            &test_ledger(),
            &roles,
            Charge::ZERO,
            &mut Work::new(PREPARATION_WORK),
            edit_roles,
        )
        .unwrap();
    assert!(matches!(
        rejected,
        RetainedNodeEditPreparation::Rejected {
            output: 23,
            denial: RetainedNodeEditDenial::CapacityExhausted { .. },
        }
    ));
    assert!(arena.hot.shares_storage_with(&original.hot));
    assert!(arena.warm.shares_storage_with(&original.warm));
    assert!(arena.cold.shares_storage_with(&original.cold));
    let consumer_cold = original.cold[1].as_deref().unwrap() as *const NodeColdData;
    let RetainedNodeEditPreparation::Ready(prepared) = arena
        .prepare_selected_epoch_nodes(
            &test_ledger(),
            &roles,
            Charge::capacity::<u8>(512 * 1024 * 1024).unwrap(),
            &mut Work::new(PREPARATION_WORK),
            edit_roles,
        )
        .unwrap()
    else {
        panic!("funded selected epoch must prepare")
    };
    assert!(arena.cold.shares_storage_with(&original.cold));
    assert!(matches!(
        prepared.install(&mut arena),
        RetainedNodeEditOutcome::Installed { output: 23, .. }
    ));
    assert_eq!(
        arena.cold[1].as_deref().unwrap() as *const NodeColdData,
        consumer_cold,
    );
    assert_eq!(arena.cold[1], original.cold[1]);
    assert!(std::sync::Arc::ptr_eq(
        &arena.warm[1].aspect_version_overrides,
        &original.warm[1].aspect_version_overrides,
    ));
    assert_eq!(arena.hot[1].as_ref().unwrap().state, NodeState::MaybeStale);
}
