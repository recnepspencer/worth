use super::super::UiMountedAppearanceFrameState;
use super::{context, context_identities, ContextIdentities};

fn projection_and_epoch() -> (
    crate::runtime::appearance::UiAppearanceProjection,
    crate::facade::WorthUiActiveApplicationSessionIdentity,
    crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    crate::facade::WorthUiActiveApplicationSession,
) {
    let (session, binding, _target, vector, theme) =
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
    (projection, session_identity, generation, session)
}

#[test]
fn persistent_fork_shares_roots_and_isolates_one_logarithmic_mutation() {
    let (projection, session_identity, generation, session) = projection_and_epoch();
    let identities = context_identities();
    let mut predecessor = UiMountedAppearanceFrameState::default();
    predecessor.begin_epoch(session_identity, &generation, &[]);
    for ordinal in 0..32 {
        let retained = context(
            ContextIdentities {
                instance: worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound()
                    .unwrap(),
                ..identities
            },
            session_identity,
            &generation,
            ordinal,
        );
        predecessor.retain_projection_for_test(&retained, projection.clone());
    }
    let mut successor = UiMountedAppearanceFrameState::default();
    successor.inherit_from(Some(&predecessor));
    assert!(successor.membership_roots_shared_with(&predecessor));
    let predecessor_counts = predecessor.membership_counts();
    let work_before = successor.membership_work();

    let added = context(
        ContextIdentities {
            instance: worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
            ..identities
        },
        session_identity,
        &generation,
        100,
    );
    successor.reserve(&added).unwrap();

    assert_eq!(predecessor.membership_counts(), predecessor_counts);
    assert_eq!(successor.membership_counts(), (32, 1, 0));
    assert!(!successor.membership_roots_shared_with(&predecessor));
    let work_after = successor.membership_work();
    assert!(work_after.1.saturating_sub(work_before.1) <= 128);
    assert_eq!(work_after.2, 0);
    let _ = session.shutdown();
}

#[test]
fn foreign_epoch_denial_does_not_reserve_or_stage_over_predecessor() {
    let (projection, session_identity, generation, session) = projection_and_epoch();
    let identities = context_identities();
    let valid = context(identities, session_identity, &generation, 1);
    let foreign_source =
        crate::runtime::tests::appearance_component_session_test_support::source_backed_static_paint_consumer_session();
    let foreign_generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
        session_identity,
        foreign_source.generation_identity(),
    );
    let foreign = context(identities, session_identity, &foreign_generation, 1);
    let mut state = UiMountedAppearanceFrameState::default();
    state.begin_epoch(session_identity, &generation, &[]);
    state.retain_projection_for_test(&valid, projection);
    let before = state.membership_counts();

    assert_eq!(
        state.reserve(&foreign),
        Err(crate::mounting::projection::UiMountedAppearanceStateMutationDenial::LocalIdentityMismatch)
    );
    assert_eq!(
        state.stage(crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
            foreign,
            crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
        )),
        Err(crate::mounting::projection::UiMountedAppearanceStateMutationDenial::LocalIdentityMismatch)
    );
    assert_eq!(state.membership_counts(), before);
    assert!(state.retained_entry_for_test(&valid).is_some());
    let _ = foreign_source.shutdown();
    let _ = session.shutdown();
}
