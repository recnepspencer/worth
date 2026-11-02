use super::*;
#[test]
fn freeing_a_unique_aspect_and_taking_it_on_another_record_checks_final_state_in_both_orders() {
    for create_taker in [false, true] {
        for free_first in [false, true] {
            let mut runtime = schema::runtime();
            let request = world::request();
            let execution = ExecutionRequest::serial(&request);
            let seed = world::commit(
                &mut runtime,
                &execution,
                vec![
                    world::create("a", "token", 7),
                    world::create("b", "other", 9),
                ],
            )
            .unwrap();
            let a = seed.created_entity(&world::created("a")).unwrap();
            let b = seed.created_entity(&world::created("b")).unwrap();
            let take = if create_taker {
                world::create("c", "token", 9)
            } else {
                world::take(b)
            };
            let mut intents = vec![world::clear(a, &["Alias"])];
            intents.insert(if free_first { 1 } else { 0 }, take);
            let result = world::commit(&mut runtime, &execution, intents).unwrap();
            let taker = if create_taker {
                result.created_entity(&world::created("c")).unwrap()
            } else {
                b
            };
            assert_eq!(
                world::field(&runtime, &execution, &result.snapshot, a, "Alias", "Unique"),
                None
            );
            assert_eq!(
                world::field(&runtime, &execution, &result.snapshot, a, "Marker", "Count"),
                Some(AspectValue::UInt64(7))
            );
            assert_eq!(
                world::field(
                    &runtime,
                    &execution,
                    &result.snapshot,
                    taker,
                    "Alias",
                    "Unique"
                ),
                Some(AspectValue::String("token".into()))
            );
            assert_eq!(
                world::field(
                    &runtime,
                    &execution,
                    &result.snapshot,
                    taker,
                    "Marker",
                    "Count"
                ),
                Some(AspectValue::UInt64(9))
            );
            if create_taker {
                assert_eq!(
                    world::field(&runtime, &execution, &result.snapshot, b, "Alias", "Unique"),
                    Some(AspectValue::String("other".into()))
                );
            }
            runtime
                .snapshots()
                .release_snapshot(&result.snapshot)
                .unwrap();
            runtime
                .snapshots()
                .release_snapshot(&seed.snapshot)
                .unwrap();
        }
    }
}
#[test]
fn a_composed_patch_and_delete_on_one_record_refuse_in_both_orders_without_publication() {
    for delete_first in [false, true] {
        let mut runtime = schema::runtime();
        let request = world::request();
        let execution = ExecutionRequest::serial(&request);
        let seed = world::commit(
            &mut runtime,
            &execution,
            vec![world::create("a", "token", 7)],
        )
        .unwrap();
        let a = seed.created_entity(&world::created("a")).unwrap();
        let mut intents = vec![world::clear(a, &["Alias", "Marker"])];
        intents.insert(
            if delete_first { 0 } else { 1 },
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: a,
            })),
        );
        let denial = world::commit(&mut runtime, &execution, intents).unwrap_err();
        let TransactionCommitError::Conflict { error, .. } = denial else {
            panic!("{denial:?}")
        };
        // A is the sole target of both intents. Query's paired owner court
        // independently checks the complete EntityIds retained in its batch.
        assert!(
            matches!(error.class, ConflictClass::ConflictingIntent { .. }),
            "{error:?}"
        );
        let current = world::unchanged_head(&runtime, &seed);
        assert_eq!(
            world::field(&runtime, &execution, &current, a, "Alias", "Unique"),
            Some(AspectValue::String("token".into()))
        );
        assert_eq!(
            world::field(&runtime, &execution, &current, a, "Marker", "Count"),
            Some(AspectValue::UInt64(7))
        );
        runtime.snapshots().release_snapshot(&current).unwrap();
        runtime
            .snapshots()
            .release_snapshot(&seed.snapshot)
            .unwrap();
    }
}

#[test]
fn taking_without_freeing_is_refused_by_the_installed_native_unique_invariant() {
    let mut runtime = schema::runtime();
    let request = world::request();
    let execution = ExecutionRequest::serial(&request);
    let seed = world::commit(
        &mut runtime,
        &execution,
        vec![
            world::create("a", "token", 7),
            world::create("b", "other", 9),
        ],
    )
    .unwrap();
    let a = seed.created_entity(&world::created("a")).unwrap();
    let b = seed.created_entity(&world::created("b")).unwrap();
    let denial = world::commit(&mut runtime, &execution, vec![world::take(b)]).unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = denial else {
        panic!("{denial:?}")
    };
    let ConflictClass::InvariantViolation { fields, .. } = error.class else {
        panic!("{error:?}");
    };
    assert_eq!(
        fields,
        InvariantViolationFields::UniqueEntityField {
            field_locator: schema::locator("Alias", "Unique"),
            value: AspectValue::String("token".into()),
        }
    );
    let current = world::unchanged_head(&runtime, &seed);
    assert_eq!(
        world::field(&runtime, &execution, &current, a, "Alias", "Unique"),
        Some(AspectValue::String("token".into()))
    );
    assert_eq!(
        world::field(&runtime, &execution, &current, b, "Alias", "Unique"),
        Some(AspectValue::String("other".into()))
    );
    runtime.snapshots().release_snapshot(&current).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&seed.snapshot)
        .unwrap();
}
