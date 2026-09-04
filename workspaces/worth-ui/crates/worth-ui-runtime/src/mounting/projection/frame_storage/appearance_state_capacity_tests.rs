use super::super::{UiMountedAppearanceFrameState, APPEARANCE_STATE_CAPACITY};
use super::{context, context_identities};

#[test]
fn appearance_state_capacity_preserves_predecessor_at_4097th_membership() {
    let (session, binding, target, vector, theme) =
        crate::runtime::appearance::projection_test_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let session_identity = session.session_identity();
    let generation = session.active_generation_identity();
    let identities = context_identities();
    let retained = context(
        identities,
        session_identity,
        &generation,
        target.graph_node().digest(),
    );
    let mut predecessor = UiMountedAppearanceFrameState::default();
    predecessor.retain_projection_for_test(&retained, projection);
    let mut state = UiMountedAppearanceFrameState::default();
    state.inherit_from(Some(&predecessor));

    for ordinal in 1..u64::try_from(APPEARANCE_STATE_CAPACITY).unwrap() {
        state
            .stage(
                crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
                    context(identities, session_identity, &generation, ordinal),
                    crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                ),
            )
            .unwrap();
    }
    state
        .reserve(&context(identities, session_identity, &generation, 1))
        .unwrap();
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
                context(identities, session_identity, &generation, 4_096),
                crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
            ),
        )
        .unwrap_err();
    assert_eq!(
        state.membership_counts(),
        (1, 0, APPEARANCE_STATE_CAPACITY - 1)
    );

    let error = state.capacity_error().expect("capacity denial is retained");
    assert_eq!(
        error,
        super::super::UiAppearanceStateCapacityExceeded { capacity: 4_096 }
    );
    assert_eq!(error.capacity(), 4_096);
    assert_eq!(
        state.membership_counts(),
        (1, 0, APPEARANCE_STATE_CAPACITY - 1)
    );
    assert_eq!(state.capacity_error(), Some(error));
    assert_eq!(
        state
            .retained_entry_for_test(&retained)
            .expect("predecessor remains retained")
            .key,
        super::super::state_key(&retained)
    );

    let _ = session.shutdown();
}

#[test]
fn exact_membership_keeps_same_session_generations_distinct_on_hash_collision() {
    use std::hash::{Hash, Hasher};

    let (session, binding, target, vector, theme) =
        crate::runtime::appearance::projection_test_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let successor_source = crate::runtime::tests::appearance_component_session_test_support::
        source_backed_static_paint_consumer_session();
    let session_identity = session.session_identity();
    let first_generation = session.active_generation_identity();
    let second_generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
        session_identity,
        successor_source.generation_identity(),
    );
    assert_ne!(first_generation, second_generation);
    let mut first_hash = std::collections::hash_map::DefaultHasher::new();
    first_generation.hash(&mut first_hash);
    let mut second_hash = std::collections::hash_map::DefaultHasher::new();
    second_generation.hash(&mut second_hash);
    assert_eq!(first_hash.finish(), second_hash.finish());

    let identities = context_identities();
    let first = context(
        identities,
        session_identity,
        &first_generation,
        target.graph_node().digest(),
    );
    let second = context(
        identities,
        session_identity,
        &second_generation,
        target.graph_node().digest(),
    );
    let mut state = UiMountedAppearanceFrameState::default();
    state.retain_projection_for_test(&first, projection.clone());
    state.retain_projection_for_test(&second, projection);
    assert_eq!(state.membership_counts(), (2, 0, 0));
    assert!(state.retained_entry_for_test(&first).is_some());
    assert!(state.retained_entry_for_test(&second).is_some());

    let _ = successor_source.shutdown();
    let _ = session.shutdown();
}
