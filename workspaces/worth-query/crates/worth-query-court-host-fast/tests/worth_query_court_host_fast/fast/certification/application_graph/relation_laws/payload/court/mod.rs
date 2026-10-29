pub(crate) use worth_query_worlds::facade::certification::relation_laws::payload::court::*;

#[test]
fn typed_relation_initialization_publishes_native_payload_and_refuses_other_programs_and_duplicates(
) {
    let world = install::<Payload>(3, &[]);
    committed(world.edit(1, &[(0, 1)]));
    let original = payloads(&world);
    assert_eq!(original.len(), 1);
    assert_eq!((original[0].0, original[0].1, original[0].3), (0, 1, 1));
    for (serial, expected) in [
        (
            2,
            WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
        ),
        (
            3,
            WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
        ),
        (
            4,
            WorthQueryApplicationAttemptDenialKind::DuplicateEffectKey,
        ),
    ] {
        let error = world
            .edit_in_scope(serial, &[(1, 2)], &request_scope())
            .unwrap_err();
        let RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause)) = error else {
            panic!("actual typed writer denial required: {error:?}");
        };
        let cause = cause
            .downcast::<WorthQueryApplicationAttemptDenial>()
            .expect("original typed writer cause");
        assert_eq!(cause.kind(), expected);
        assert_eq!(payloads(&world), original);
    }
    assert!(matches!(
        world.edit(1, &[(0, 1)]),
        Outcome::AlreadyCommitted(_) | Outcome::PreviouslyCommitted(_)
    ));
    assert!(matches!(
        world.edit(1, &[(0, 2)]),
        Outcome::IdempotencyIntentDrift
    ));
    assert_eq!(
        payloads(&world),
        original,
        "command drift cannot initialize an existing relation"
    );
    committed(world.edit(5, &[(1, 2)]));
    let actual = payloads(&world);
    assert_eq!(actual.len(), 2);
    assert_eq!(actual[0], original[0]);
    assert_eq!((actual[1].0, actual[1].1, actual[1].3), (1, 2, 5));
    assert_ne!(
        actual[0].2, actual[1].2,
        "genuine physical relation occurrences"
    );
    world.close();
}
