use super::super::{UiMountedAppearanceFrameState, APPEARANCE_STATE_CAPACITY};
use super::{context, context_identities};
use crate::mounting::projection::appearance::UiMountedAppearanceGeometryScope;

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

#[test]
fn physical_only_membership_keeps_capacity_across_expiry_and_denial() {
    let (session, binding, _, vector, theme) = crate::runtime::appearance::projection_test_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let fixture =
        crate::mounting::projection::appearance::mounted_sidecar_with_retained_facts_for_test();
    let generation = session.active_generation_identity();
    let retained = context(
        super::ContextIdentities {
            frame: fixture.frame,
            issuer: fixture.issuer,
            instance: fixture.instance,
            surface: fixture.surface,
            incarnation: worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap(),
        },
        session.session_identity(),
        &generation,
        fixture.graph_node.digest(),
    );
    let mut state = UiMountedAppearanceFrameState::default();
    state.begin_epoch(session.session_identity(), &generation, &[]);
    state.retain_projection_for_test(&retained, projection);
    state.replace_sidecar_for_test(&retained, fixture.sidecar);
    for _ in 0..3 {
        state.members.clear_for_epoch();
        assert!(state.retained_entry_for_test(&retained).is_none());
        assert_eq!(
            state.physical_node_receipts_for_test(),
            vec![fixture.receipt]
        );
    }
    for ordinal in 1..APPEARANCE_STATE_CAPACITY {
        let reserved = context(
            context_identities(),
            session.session_identity(),
            &generation,
            ordinal as u64,
        );
        state.reserve(&reserved).unwrap();
    }
    let overflow = context(
        context_identities(),
        session.session_identity(),
        &generation,
        4_097,
    );
    assert!(matches!(
        state.reserve(&overflow),
        Err(super::super::UiMountedAppearanceStateMutationDenial::Capacity(_))
    ));
    assert_eq!(
        state.physical_node_receipts_for_test(),
        vec![fixture.receipt]
    );
    // Replacement reuses the occupied slot; denied semantic work restores the
    // physical-only predecessor without granting the expired projection.
    let denied = crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
        retained.clone(),
        crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
    );
    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    for _ in 0..3 {
        state.stage(denied.clone()).unwrap();
        let records = state
            .lower(
                presentation,
                &UiMountedAppearanceGeometryScope::new(&[], None),
            )
            .unwrap();
        assert_eq!(records.len(), 1);
        assert!(matches!(
            records[0],
            crate::runtime::appearance::UiAppearanceInspectionRecord::Denial { .. }
        ));
        assert_eq!(
            state.physical_node_receipts_for_test(),
            vec![fixture.receipt]
        );
        assert!(state.retained_entry_for_test(&retained).is_none());
    }
    state.prepare_reconstruction(Vec::new());
    assert_eq!(state.lower(presentation, &UiMountedAppearanceGeometryScope::new(&[], None)).err().unwrap(), super::super::super::appearance_output::UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable);
    assert_eq!(
        state.physical_node_receipts_for_test(),
        vec![fixture.receipt]
    );
    let _ = session.shutdown();
}
