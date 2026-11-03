//! Exact local posting grouping and reverse-index witnesses.

pub use std::sync::Arc;

pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::identity::KindId;
pub use worth_relational::facade::identity::PartitionId;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::fact_key::FactPostingKey;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::MarkState;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::postings::group_projected;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::postings::PreparedPostingOrdinals;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::postings::ProjectedKey;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;

#[test]
fn equal_fingerprints_keep_distinct_exact_keys_and_merge_only_equal_keys() {
    let first = Arc::new(FactPostingKey::EntityLifecycle(EntityId::new(
        PartitionId::main(),
        1,
        1,
    )));
    let second = Arc::new(FactPostingKey::EntityLifecycle(EntityId::new(
        PartitionId::main(),
        2,
        1,
    )));
    let projected = vec![
        ProjectedKey {
            hash: 7,
            key: Arc::clone(&first),
            ordinal: 0,
        },
        ProjectedKey {
            hash: 7,
            key: Arc::clone(&second),
            ordinal: 1,
        },
        ProjectedKey {
            hash: 7,
            key: Arc::clone(&first),
            ordinal: 2,
        },
    ];

    let groups = group_projected(projected).unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].key.as_ref(), first.as_ref());
    assert_eq!(
        groups[0].ordinals.iter().copied().collect::<Vec<_>>(),
        vec![0, 2]
    );
    assert_eq!(groups[1].key.as_ref(), second.as_ref());
    assert_eq!(
        groups[1].ordinals.iter().copied().collect::<Vec<_>>(),
        vec![1]
    );
}

#[test]
fn source_and_output_facts_share_one_key_but_keep_both_composite_ordinals() {
    let (_, identity) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let source = [Fact::SourceEntity { entity_id: entity }];
    let output = [Fact::Entity {
        entity_id: entity,
        kind: KindId::new(9),
    }];

    let prepared = PreparedPostingOrdinals::prepare(&source, Some(&output), &mut None).unwrap();
    let mut state = MarkState::initial();
    let projection = prepared.into_projection(0, source.len(), output.len(), None);
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::posting_bindings::bind(&mut state, &projection, &identity).unwrap();
    let ordinals = &projection.ordinals;
    let key = FactPostingKey::EntityLifecycle(entity);
    let own = ordinals.get(&key).unwrap();
    assert_eq!(own.iter().copied().collect::<Vec<_>>(), vec![0, 1]);
    let global = state.postings.get(&key).unwrap();
    assert_eq!(global.iter().copied().collect::<Vec<_>>(), vec![0]);
    let binding = state.posting_bindings.get(&0).unwrap();
    let binding = binding.settlements().unwrap();
    assert_eq!(binding.settlements.len(), 1);
    assert!(binding.settlements.contains(&identity));
}

#[test]
fn final_posting_storage_keeps_all_ordinals_without_retaining_grouping_scratch() {
    for (keys, copies) in [(1usize, 1usize), (1, 128), (128, 1)] {
        let facts: Vec<_> = (0..keys)
            .flat_map(|key| {
                std::iter::repeat_n(
                    Fact::SourceEntity {
                        entity_id: EntityId::new(PartitionId::main(), key as u64 + 1, 1),
                    },
                    copies,
                )
            })
            .collect();

        let prepared = PreparedPostingOrdinals::prepare(&facts, None, &mut None).unwrap();
        assert_eq!(prepared.ordinals.len(), keys);
        assert!(prepared
            .ordinals
            .values()
            .all(|ordinals| ordinals.len() == copies));

        let (_, identity) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
        let mut state = MarkState::initial();
        let prior = state.postings.clone();
        let projection = prepared.into_projection(0, facts.len(), 0, None);

        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::posting_bindings::bind(&mut state, &projection, &identity).unwrap();
        assert!(prior.is_empty());
        assert_eq!(state.posting_count, keys);
        assert_eq!(state.postings.len(), keys);
        assert!(state
            .postings
            .values()
            .all(|groups| groups.iter().copied().collect::<Vec<_>>() == vec![0]));
        drop(state);

        drop(projection);
    }
}

/// Query's posting store and native marking produce Relational's two
/// preflight counter-overflow stops; Relational defers them verbatim.
#[test]
fn posting_and_marking_counters_stop_with_their_typed_overflow() {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::delivery::affected::mark_keys;
    use worth_query_execution::facade::domain_computation::WorthQueryNativeMarkingWork;
    use worth_relational::facade::mvcc::CompanionPreflightStop;

    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let facts = [Fact::SourceEntity { entity_id: entity }];
    let projection = PreparedPostingOrdinals::prepare(&facts, None, &mut None)
        .unwrap()
        .into_projection(0, facts.len(), 0, None);
    let (_, identity) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();

    let mut full = MarkState::initial();
    full.posting_count = usize::MAX;
    assert_eq!(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::posting_bindings::bind(&mut full, &projection, &identity),
        Err(CompanionPreflightStop::PreparationMemoryCounterOverflow)
    );

    let mut state = MarkState::initial();
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::posting_bindings::bind(&mut state, &projection, &identity).unwrap();
    let mut counts = WorthQueryNativeMarkingWork {
        posting_key_lookups: u64::MAX,
        ..WorthQueryNativeMarkingWork::default()
    };
    assert_eq!(
        mark_keys(
            &mut state,
            &[FactPostingKey::EntityLifecycle(entity)],
            &mut counts
        )
        .map(|affected| affected.len()),
        Err(CompanionPreflightStop::WorkCounterOverflow)
    );
}

#[path = "bindings/mod.rs"]
pub mod bindings;
