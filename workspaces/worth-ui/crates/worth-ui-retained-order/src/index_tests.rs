use super::BoundedOrderIndex;

#[test]
fn adversarial_mutations_match_an_independent_vector_model() {
    exercise_model();
}

fn exercise_model() {
    let mut index = BoundedOrderIndex::new(8_192);
    let mut model = Vec::new();
    let mut next_identity = 0_u64;
    let mut state = 0x8d26_4b91_7ac3_05e1_u64;
    for step in 0..8_192 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        if step % 3 == 0 && !model.is_empty() {
            let removed = usize::try_from(state).unwrap() % model.len();
            assert!(index.remove(model.remove(removed)));
        } else {
            let rank = usize::try_from(state).unwrap() % (model.len() + 1);
            model.insert(rank, next_identity);
            index.insert_at(rank, next_identity).unwrap();
            next_identity += 1;
        }
        let operation = index.take_cost();
        let logarithmic_touch_bound = 8 * u64::from((model.len() + 2).ilog2()) + 16;
        assert!(operation.node_touches() <= logarithmic_touch_bound);
        assert!(operation.rotations() <= 2 * u64::from((model.len() + 2).ilog2()) + 2);
        assert_eq!(
            operation.live_entries(),
            u64::try_from(model.len()).unwrap()
        );
        assert!(operation.allocated_slots() <= operation.high_water_entries());
        if step % 37 == 0 {
            assert_model(&index, &model);
            index.take_cost();
        }
    }
    assert_model(&index, &model);
    let logarithmic_bound = 2 * usize::try_from((model.len() + 1).ilog2()).unwrap() + 2;
    assert!(index.height() <= logarithmic_bound);
}

#[test]
fn capacity_denial_preserves_sequence_and_cost_state() {
    const CAPACITY: usize = 4_096;
    let mut index = BoundedOrderIndex::new(CAPACITY);
    for identity in 0..u64::try_from(CAPACITY).unwrap() {
        index.insert_at(identity as usize, identity).unwrap();
    }
    let expected = index.ordered().collect::<Vec<_>>();
    index.take_cost();
    assert_eq!(
        index.insert_at(CAPACITY / 2, u64::try_from(CAPACITY).unwrap()),
        Err(crate::UiRetainedOrderDenial::CapacityExceeded)
    );
    let denied = index.take_cost();
    assert_eq!(denied.identity_lookups(), 0);
    assert_eq!(denied.node_touches(), 0);
    assert_eq!(denied.rotations(), 0);
    assert_eq!(denied.live_entries(), u64::try_from(CAPACITY).unwrap());
    assert_eq!(denied.allocated_slots(), u64::try_from(CAPACITY).unwrap());
    assert_eq!(
        denied.high_water_entries(),
        u64::try_from(CAPACITY).unwrap()
    );
    assert_eq!(index.ordered().collect::<Vec<_>>(), expected);
    let cost = index.take_cost();
    assert_eq!(cost.live_entries(), u64::try_from(CAPACITY).unwrap());
    assert_eq!(cost.allocated_slots(), u64::try_from(CAPACITY).unwrap());
    assert_eq!(cost.high_water_entries(), u64::try_from(CAPACITY).unwrap());
    assert_eq!(cost.rotations(), 0);
}

#[test]
fn weighted_successor_search_follows_sequence_order_across_updates() {
    let mut index = BoundedOrderIndex::new(8);
    let mut model = vec![(10_u64, 8_u32), (11, 2), (12, 9), (13, 4)];
    for (rank, (identity, weight)) in model.iter().copied().enumerate() {
        index.insert_at_weighted(rank, identity, weight).unwrap();
    }
    assert_weighted_search(&index, &model);

    assert!(index.update_weight(11, 12));
    model[1].1 = 12;
    assert!(index.remove(10));
    model.remove(0);
    index.insert_at_weighted(2, 14, 7).unwrap();
    model.insert(2, (14, 7));
    assert_weighted_search(&index, &model);
}

fn assert_weighted_search(index: &BoundedOrderIndex<u64>, model: &[(u64, u32)]) {
    for minimum in 0..=13 {
        assert_eq!(
            index.first_with_weight_at_least(minimum),
            model
                .iter()
                .find(|(_, weight)| *weight >= minimum)
                .map(|(identity, _)| *identity)
        );
    }
}

fn assert_model(index: &BoundedOrderIndex<u64>, model: &[u64]) {
    assert_eq!(index.ordered().collect::<Vec<_>>(), model);
    for (rank, identity) in model.iter().copied().enumerate() {
        assert_eq!(index.rank(identity), Some(rank));
        assert_eq!(index.identity_at(rank), Some(identity));
    }
}
