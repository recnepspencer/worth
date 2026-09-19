use super::retained_node_edit::{
    RetainedNodeEditDenial, RetainedNodeEditOutcome, RetainedNodeEditPreparation,
    RetainedNodePayload,
};
use super::{NodeArena, SignalGraph};
mod clone_work;
mod publication_peak;
mod reservation_lifetime;
use crate::data::node::{NodeColdData, NodeHotData, NodeState, NodeWarmData};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement, RetainedStoragePreparation as Work,
};

// Structural page-write work is now included in preparation. Capacity tests
// use a generous probe; exact/short work tests derive their limits from it.
const PREPARATION_WORK: usize = 100_000;

fn test_ledger() -> std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionLedger>
{
    crate::data::retained_storage::SignalConditionalRetentionLedger::new(
        crate::runtime_policy::SignalConditionalEvaluationBudget {
            maximum_retained_slots: 8,
            maximum_retained_bytes: 512 * 1024 * 1024,
            maximum_attempt_visits: PREPARATION_WORK,
        },
        crate::runtime_policy::SignalRuntimePolicy::development().conditional_temporal_budget,
    )
}

fn prepared_arena() -> NodeArena {
    let mut graph = SignalGraph::new();
    graph.create_node();
    graph.create_node();
    graph.arena.hot = graph.arena.hot.fork_persistent();
    graph.arena.warm = graph.arena.warm.fork_persistent();
    graph.arena.cold = graph.arena.cold.fork_persistent();
    graph
        .arena
        .hot
        .prepare_retained_charge(&mut Work::new(1_000))
        .unwrap();
    graph
        .arena
        .warm
        .prepare_retained_charge(&mut Work::new(1_000))
        .unwrap();
    graph
        .arena
        .cold
        .prepare_retained_charge(&mut Work::new(1_000))
        .unwrap();
    graph.arena
}

fn edit_payloads(
    hot: &mut NodeHotData,
    warm: &mut NodeWarmData,
    cold: &mut Option<Box<NodeColdData>>,
) -> u32 {
    hot.state = NodeState::Clean;
    warm.direct_invalidation_generation = 7;
    *cold = Some(Box::new(NodeColdData {
        causality: Some(crate::data::trace::CausalityMetadata {
            kind: "large retained cause".repeat(1_024),
            fields: Default::default(),
        }),
        ..Default::default()
    }));
    17
}

fn edit_selected_payloads(payloads: &mut [RetainedNodePayload]) -> u32 {
    for payload in payloads {
        edit_payloads(&mut payload.hot, &mut payload.warm, &mut payload.cold);
    }
    17
}

#[test]
fn cumulative_same_page_edits_admit_and_install_all_selected_nodes_together() {
    let mut arena = prepared_arena();
    let original = arena.clone();
    let mut expected = arena.clone();
    for index in 0..2 {
        edit_payloads(
            expected.hot[index].as_mut().unwrap(),
            &mut expected.warm[index],
            &mut expected.cold[index],
        );
    }
    let maximum = representation_charge(&expected);
    let short = maximum
        .checked_sub(Charge::capacity::<u8>(1).unwrap())
        .unwrap();
    let mut work = Work::new(100_000);
    assert!(
        matches!(arena.prepare_retained_node_edits(&test_ledger(), &[0, 1], short, &mut work,
        edit_selected_payloads).unwrap(), RetainedNodeEditPreparation::Rejected {
            output: 17, denial: RetainedNodeEditDenial::CapacityExhausted { required, .. }
        } if required == maximum)
    );
    assert!(matches!(
        arena
            .prepare_retained_node_edits(
                &test_ledger(),
                &[0, 1],
                maximum,
                &mut Work::new(work.visits() - 1),
                edit_selected_payloads
            )
            .unwrap(),
        RetainedNodeEditPreparation::Rejected {
            output: 17,
            denial: RetainedNodeEditDenial::Accounting(
                crate::data::retained_storage::RetainedStoragePreparationDenial::WorkExhausted { .. }
            )
        }
    ));
    assert!(arena.hot.shares_storage_with(&original.hot));
    assert!(arena.warm.shares_storage_with(&original.warm));
    assert!(arena.cold.shares_storage_with(&original.cold));
    let RetainedNodeEditPreparation::Ready(prepared) = arena
        .prepare_retained_node_edits(
            &test_ledger(),
            &[0, 1],
            maximum,
            &mut Work::new(work.visits()),
            edit_selected_payloads,
        )
        .unwrap()
    else {
        panic!("exact cumulative capacity and work must prepare")
    };
    assert!(arena.hot.shares_storage_with(&original.hot));
    assert!(arena.warm.shares_storage_with(&original.warm));
    assert!(arena.cold.shares_storage_with(&original.cold));
    assert!(matches!(prepared.install(&mut arena),
        RetainedNodeEditOutcome::Installed { output: 17, charge } if charge == maximum));
    assert_eq!(arena.hot, expected.hot);
    assert_eq!(arena.warm, expected.warm);
    assert_eq!(arena.cold, expected.cold);
    assert_eq!(representation_charge(&arena), maximum);
    for index in 0..2 {
        assert!(original.cold[index].is_none());
    }
}

#[test]
fn cumulative_selection_validates_every_node_before_editing() {
    let arena = prepared_arena();
    let maximum = representation_charge(&arena);
    for indices in [&[][..], &[0, 0], &[1, 0], &[0, 2]] {
        assert!(arena
            .prepare_retained_node_edits(
                &test_ledger(),
                indices,
                maximum,
                &mut Work::new(10_000),
                |_| panic!("invalid selection must fail before callback")
            )
            .is_err());
    }
}

fn representation_charge(arena: &NodeArena) -> Charge {
    let mut work = Work::new(10_000);
    arena
        .hot
        .retained_heap_charge(&mut work)
        .unwrap()
        .checked_add(arena.warm.retained_heap_charge(&mut work).unwrap())
        .unwrap()
        .checked_add(arena.cold.retained_heap_charge(&mut work).unwrap())
        .unwrap()
}

#[test]
fn delayed_retained_node_install_rejects_intervening_page_and_extent_changes() {
    for truncate in [false, true] {
        let mut arena = prepared_arena();
        let mut expected = arena.clone();
        edit_payloads(
            expected.hot[0].as_mut().unwrap(),
            &mut expected.warm[0],
            &mut expected.cold[0],
        );
        let maximum = representation_charge(&expected);
        let RetainedNodeEditPreparation::Ready(prepared) = arena
            .prepare_retained_node_edits(
                &test_ledger(),
                &[0],
                maximum,
                &mut Work::new(PREPARATION_WORK),
                edit_selected_payloads,
            )
            .unwrap()
        else {
            panic!("known capacity must prepare");
        };
        assert!(prepared.storage_preconditions_hold(&arena));
        if truncate {
            arena.hot.pop_back();
            arena.warm.pop_back();
            arena.cold.pop_back();
        } else {
            let edited = arena
                .warm
                .edit_with_retained_charge(1, &mut Work::new(10_000), |warm| {
                    warm.direct_invalidation_generation = 99
                })
                .unwrap();
            assert!(matches!(
                edited,
                crate::data::persistent_vector::RetainedVectorMutationOutcome::Accounted { .. }
            ));
        }
        let current_hot = arena.hot.clone();
        let current_warm = arena.warm.clone();
        let current_cold = arena.cold.clone();
        assert!(!prepared.storage_preconditions_hold(&arena));
        assert!(matches!(
            prepared.install(&mut arena),
            RetainedNodeEditOutcome::Rejected {
                output: 17,
                denial: RetainedNodeEditDenial::StorageChanged
            }
        ));
        assert!(arena.hot.shares_storage_with(&current_hot));
        assert!(arena.warm.shares_storage_with(&current_warm));
        assert!(arena.cold.shares_storage_with(&current_cold));
        assert!(arena.cold[0].is_none());
        if truncate {
            assert_eq!(arena.warm.len(), 1);
        } else {
            assert_eq!(arena.warm[1].direct_invalidation_generation, 99);
        }
    }
}

#[test]
fn retained_node_capacity_rejects_all_lanes_together_and_installs_at_exact_bound() {
    let mut arena = prepared_arena();
    let original_generation = arena.warm[0].direct_invalidation_generation;
    let sibling = arena.clone();
    let mut expected = arena.clone();
    edit_payloads(
        expected.hot[0].as_mut().unwrap(),
        &mut expected.warm[0],
        &mut expected.cold[0],
    );
    let charge = representation_charge(&expected);
    let short = charge
        .checked_sub(Charge::capacity::<u8>(1).unwrap())
        .unwrap();
    let rejected = arena
        .prepare_retained_node_edits(
            &test_ledger(),
            &[0],
            short,
            &mut Work::new(PREPARATION_WORK),
            edit_selected_payloads,
        )
        .unwrap();
    assert!(
        matches!(rejected, RetainedNodeEditPreparation::Rejected { output: 17,
        denial: RetainedNodeEditDenial::CapacityExhausted { maximum, required } }
        if maximum == short && required == charge)
    );
    assert!(arena.hot.shares_storage_with(&sibling.hot));
    assert!(arena.warm.shares_storage_with(&sibling.warm));
    assert!(arena.cold.shares_storage_with(&sibling.cold));
    assert_eq!(
        arena.warm[0].direct_invalidation_generation,
        original_generation
    );
    assert!(arena.cold[0].is_none());
    let RetainedNodeEditPreparation::Ready(prepared) = arena
        .prepare_retained_node_edits(
            &test_ledger(),
            &[0],
            charge,
            &mut Work::new(PREPARATION_WORK),
            edit_selected_payloads,
        )
        .unwrap()
    else {
        panic!("exact capacity must prepare");
    };
    assert!(arena.hot.shares_storage_with(&sibling.hot));
    assert!(arena.warm.shares_storage_with(&sibling.warm));
    assert!(arena.cold.shares_storage_with(&sibling.cold));
    let installed = prepared.install(&mut arena);
    assert!(
        matches!(installed, RetainedNodeEditOutcome::Installed { output: 17, charge: actual } if actual == charge)
    );
    assert_eq!(arena.hot, expected.hot);
    assert_eq!(arena.warm, expected.warm);
    assert_eq!(arena.cold, expected.cold);
    assert_eq!(representation_charge(&arena), charge);
    assert_eq!(
        sibling.warm[0].direct_invalidation_generation,
        original_generation
    );
}

#[test]
fn retained_node_denial_and_unwind_preserve_every_original_lane() {
    let mut arena = prepared_arena();
    let sibling = arena.clone();
    let mut expected = arena.clone();
    edit_payloads(
        expected.hot[0].as_mut().unwrap(),
        &mut expected.warm[0],
        &mut expected.cold[0],
    );
    let maximum = representation_charge(&expected);
    let early =
        arena.prepare_retained_node_edits(&test_ledger(), &[0], maximum, &mut Work::new(0), |_| {
            panic!("zero-budget edit ran")
        });
    assert!(matches!(
        early,
        Err(RetainedNodeEditDenial::Accounting(
            crate::data::retained_storage::RetainedStoragePreparationDenial::WorkExhausted { .. }
        ))
    ));
    let sample = arena.clone();
    let mut work = Work::new(PREPARATION_WORK);
    assert!(matches!(
        sample
            .prepare_retained_node_edits(
                &test_ledger(),
                &[0],
                maximum,
                &mut work,
                edit_selected_payloads
            )
            .unwrap(),
        RetainedNodeEditPreparation::Ready(_)
    ));
    let rejected = arena
        .prepare_retained_node_edits(
            &test_ledger(),
            &[0],
            maximum,
            &mut Work::new(work.visits() - 1),
            edit_selected_payloads,
        )
        .unwrap();
    assert!(matches!(
        rejected,
        RetainedNodeEditPreparation::Rejected {
            output: 17,
            denial: RetainedNodeEditDenial::Accounting(
                crate::data::retained_storage::RetainedStoragePreparationDenial::WorkExhausted { .. }
            )
        }
    ));
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = arena.prepare_retained_node_edits(
            &test_ledger(),
            &[0],
            maximum,
            &mut Work::new(10_000),
            |payloads| {
                edit_selected_payloads(payloads);
                panic!("detached node edit failed");
            },
        );
    }));
    assert!(unwind.is_err());
    assert!(arena.hot.shares_storage_with(&sibling.hot));
    assert!(arena.warm.shares_storage_with(&sibling.warm));
    assert!(arena.cold.shares_storage_with(&sibling.cold));
    assert_eq!(
        arena.hot.prepared_retained_charge(),
        sibling.hot.prepared_retained_charge()
    );
    assert_eq!(
        arena.warm.prepared_retained_charge(),
        sibling.warm.prepared_retained_charge()
    );
    assert_eq!(
        arena.cold.prepared_retained_charge(),
        sibling.cold.prepared_retained_charge()
    );
    arena.warm[0].direct_invalidation_generation = 1;
    assert!(matches!(
        arena.prepare_retained_node_edits(
            &test_ledger(),
            &[0],
            maximum,
            &mut Work::new(10_000),
            |_| panic!("unprepared edit ran")
        ),
        Err(RetainedNodeEditDenial::Lane(_))
    ));
}
