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
    predecessor.begin_epoch(session_identity, &generation, &[]);
    predecessor.retain_projection_for_test(&retained, projection);
    let mut state = UiMountedAppearanceFrameState::default();
    state.inherit_from(Some(&predecessor));
    state.begin_epoch(session_identity, &generation, &[]);

    let first_stage = context(context_identities(), session_identity, &generation, 1);
    for ordinal in 1..u64::try_from(APPEARANCE_STATE_CAPACITY).unwrap() {
        let staged = if ordinal == 1 {
            first_stage.clone()
        } else {
            context(context_identities(), session_identity, &generation, ordinal)
        };
        state
            .stage(
                crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
                    staged,
                    crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                ),
            )
            .unwrap();
    }
    state.reserve(&first_stage).unwrap();
    let overflow = context(context_identities(), session_identity, &generation, 4_096);
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
                overflow,
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
