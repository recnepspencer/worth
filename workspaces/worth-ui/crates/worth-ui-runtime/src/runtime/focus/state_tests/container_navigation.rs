use super::*;

#[test]
fn active_descendant_moves_without_moving_semantic_focus_and_clears_on_composite_exit() {
    let (snapshot, scope, container, children, ordinary) = container_world(
        crate::capability::ComponentFocusSupport::active_descendant_focus_container(
            crate::capability::ComponentFocusNavigationAxis::Horizontal,
            true,
        ),
    );
    let mut state = UiFocusRuntimeState::new_session_restore_candidate();
    state.reconcile_mounted_participation(&snapshot).unwrap();
    state
        .commit(state.plan(first_request(scope)).unwrap())
        .unwrap();

    let before = state.inspect_for_certification();
    assert!(state
        .navigate_container(
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            crate::runtime::focus::UiFocusContainerNavigationKey::Right,
        )
        .unwrap()
        .is_none());
    assert_eq!(state.inspect_for_certification(), before);

    assert!(matches!(
        state
            .navigate_container(
                scope.semantic_surface(),
                crate::runtime::focus::UiFocusContainerNavigationKey::Right
            )
            .unwrap(),
        Some(crate::runtime::focus::UiFocusContainerNavigationReceipt::ActiveDescendant)
    ));
    let active = state.inspect().active_descendant().unwrap();
    assert_eq!(active.descendant(), children[0].0);
    assert_eq!(
        state.inspect().current().unwrap().participant(),
        container.0
    );
    assert_eq!(
        state.inspect().accessibility_focus(),
        crate::runtime::focus::UiAccessibilityFocusHookSupport::UnsupportedUntilMilestone13
    );

    state
        .commit(
            state
                .plan(UiFocusRequest::Direct {
                    scope,
                    participant: ordinary.0,
                    incarnation: ordinary.1,
                    cause: UiFocusCause::Direct,
                })
                .unwrap(),
        )
        .unwrap();
    assert_eq!(state.inspect().active_descendant(), None);
}

#[test]
fn roving_policy_moves_semantic_focus_and_tab_leaves_the_container_as_one_stop() {
    let (snapshot, scope, _container, children, ordinary) = container_world(
        crate::capability::ComponentFocusSupport::roving_focus_container(
            crate::capability::ComponentFocusNavigationAxis::Vertical,
            false,
        ),
    );
    let mut state = UiFocusRuntimeState::new_session_restore_candidate();
    state.reconcile_mounted_participation(&snapshot).unwrap();
    state
        .commit(state.plan(first_request(scope)).unwrap())
        .unwrap();

    let before = state.inspect_for_certification();
    assert!(state
        .navigate_container(
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            crate::runtime::focus::UiFocusContainerNavigationKey::Down,
        )
        .unwrap()
        .is_none());
    assert_eq!(state.inspect_for_certification(), before);

    let first = state
        .navigate_container(
            scope.semantic_surface(),
            crate::runtime::focus::UiFocusContainerNavigationKey::Down,
        )
        .unwrap();
    let Some(crate::runtime::focus::UiFocusContainerNavigationReceipt::Roving(first)) = first
    else {
        panic!("declared vertical roving navigation must move semantic focus");
    };
    assert_eq!(first.current().unwrap().participant(), children[0].0);
    assert_eq!(first.cause(), UiFocusCause::RovingMovement);
    assert!(state
        .navigate_container(
            scope.semantic_surface(),
            crate::runtime::focus::UiFocusContainerNavigationKey::Right
        )
        .unwrap()
        .is_none());

    let outside = state
        .commit_host_traversal(
            scope,
            crate::runtime::focus::UiHostFocusTraversalDirection::Forward,
            false,
        )
        .unwrap();
    assert_eq!(outside.current().unwrap().participant(), ordinary.0);
}

fn container_world(
    support: crate::capability::ComponentFocusSupport,
) -> (
    crate::mounting::UiMountedFocusParticipationSnapshot,
    UiFocusScopeIdentity,
    (
        UiFocusParticipantIdentity,
        worth_ui_host_contract::UiMountIncarnation,
    ),
    [(
        UiFocusParticipantIdentity,
        worth_ui_host_contract::UiMountIncarnation,
    ); 2],
    (
        UiFocusParticipantIdentity,
        worth_ui_host_contract::UiMountIncarnation,
    ),
) {
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let mounted = (0..4)
        .map(|_| worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap())
        .collect::<Vec<_>>();
    let incarnations = (0..4)
        .map(|_| worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap())
        .collect::<Vec<_>>();
    let make = |index: usize, support| {
        crate::mounting::UiMountedFocusParticipant::new(
            crate::graph::UiGraphNodeIdentity::new(index as u64 + 1),
            surface,
            mounted[index],
            incarnations[index],
            issuer.receipt_for(mounted[index]),
            support,
            crate::mounting::UiMountedFocusScope::ActiveSurface,
            index as u32,
        )
    };
    let participants = vec![
        make(0, support),
        make(1, crate::capability::ComponentFocusSupport::focusable()).with_container(mounted[0]),
        make(2, crate::capability::ComponentFocusSupport::focusable()).with_container(mounted[0]),
        make(3, crate::capability::ComponentFocusSupport::focusable()),
    ];
    let identity = |index| {
        (
            UiFocusParticipantIdentity::for_mounted_instance(mounted[index]),
            incarnations[index],
        )
    };
    (
        crate::mounting::UiMountedFocusParticipationSnapshot::new(frame, participants, 4),
        UiFocusScopeIdentity::for_surface(surface),
        identity(0),
        [identity(1), identity(2)],
        identity(3),
    )
}
