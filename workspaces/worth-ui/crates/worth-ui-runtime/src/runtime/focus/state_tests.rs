use super::UiFocusRuntimeState;
use crate::runtime::focus::{
    UiFocusCause, UiFocusOutcome, UiFocusParticipantIdentity, UiFocusRequest, UiFocusScopeIdentity,
    UiFocusTraversalDirection,
};

#[path = "state_tests/container_navigation.rs"]
mod container_navigation;
#[path = "state_tests/policy_defaults.rs"]
mod policy_defaults;
#[path = "state_tests/prepared_reconciliation.rs"]
mod prepared_reconciliation;

#[test]
fn scoped_traversal_wraps_in_mounted_order_with_constant_route_visits() {
    let world = World::new(3);
    let mut state = UiFocusRuntimeState::new_session_restore_candidate();
    let reconciliation = state
        .reconcile_mounted_participation(&world.snapshot)
        .unwrap();
    assert_eq!(reconciliation.participants_installed(), 3);
    assert_eq!(reconciliation.mounted_nodes_visited(), 3);
    state.observe_window_focus(true);

    let first = state
        .commit(state.plan(first_request(world.scope)).unwrap())
        .unwrap();
    assert_eq!(
        first.current().unwrap().participant(),
        world.identities[0].0
    );
    assert_eq!(state.last_transition(), Some(first));
    let stale = state.plan(first_request(world.scope)).unwrap();

    let forward = state
        .commit(
            state
                .plan(UiFocusRequest::Traverse {
                    scope: world.scope,
                    direction: UiFocusTraversalDirection::Forward,
                    wrap: true,
                })
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        forward.current().unwrap().participant(),
        world.identities[1].0
    );
    assert_eq!(forward.participants_visited(), 1);
    assert!(state.inspect().focus_visible());
    assert_eq!(
        state.commit(stale),
        Err(crate::runtime::focus::UiFocusRoutingDenial::StalePlan)
    );

    state
        .commit(
            state
                .plan(UiFocusRequest::Direct {
                    scope: world.scope,
                    participant: world.identities[0].0,
                    incarnation: world.identities[0].1,
                    cause: UiFocusCause::Direct,
                })
                .unwrap(),
        )
        .unwrap();
    let wrapped = state
        .commit(
            state
                .plan(UiFocusRequest::Traverse {
                    scope: world.scope,
                    direction: UiFocusTraversalDirection::Backward,
                    wrap: true,
                })
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        wrapped.current().unwrap().participant(),
        world.identities[2].0
    );
}

#[test]
fn mosaic_region_scopes_route_independently_on_one_surface() {
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let first_instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let second_instance =
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let first_region = crate::graph::UiGraphNodeIdentity::new(101);
    let second_region = crate::graph::UiGraphNodeIdentity::new(202);
    let first_scope = crate::mounting::UiMountedFocusScope::MosaicRegion {
        owner: first_region,
        kind: crate::capability::MosaicFocusScopeKind::RegionScope,
    };
    let second_scope = crate::mounting::UiMountedFocusScope::MosaicRegion {
        owner: second_region,
        kind: crate::capability::MosaicFocusScopeKind::ModalTrapScope,
    };
    let participants = vec![
        crate::mounting::UiMountedFocusParticipant::new(
            crate::graph::UiGraphNodeIdentity::new(1),
            surface,
            first_instance,
            worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap(),
            issuer.receipt_for(first_instance),
            crate::capability::ComponentFocusSupport::focusable(),
            first_scope,
            0,
        ),
        crate::mounting::UiMountedFocusParticipant::new(
            crate::graph::UiGraphNodeIdentity::new(2),
            surface,
            second_instance,
            worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap(),
            issuer.receipt_for(second_instance),
            crate::capability::ComponentFocusSupport::focusable(),
            second_scope,
            1,
        ),
    ];
    let snapshot = crate::mounting::UiMountedFocusParticipationSnapshot::new(
        frame,
        participants,
        2,
        Vec::new(),
    );
    let mut state = UiFocusRuntimeState::new_session_restore_candidate();
    state.reconcile_mounted_participation(&snapshot).unwrap();
    let first_scope = UiFocusScopeIdentity::from_mounted(surface, first_scope);
    let second_scope = UiFocusScopeIdentity::from_mounted(surface, second_scope);

    let first = state
        .commit(state.plan(first_request(first_scope)).unwrap())
        .unwrap();
    assert_eq!(
        first.current().unwrap().participant().mounted_instance(),
        first_instance
    );
    let second = state
        .commit(state.plan(first_request(second_scope)).unwrap())
        .unwrap();
    assert_eq!(
        second.current().unwrap().participant().mounted_instance(),
        second_instance
    );
    assert_ne!(first_scope, second_scope);
}

fn first_request(scope: UiFocusScopeIdentity) -> UiFocusRequest {
    UiFocusRequest::First {
        scope,
        cause: UiFocusCause::Direct,
    }
}

#[test]
fn reconciliation_never_restores_an_equal_identity_with_a_foreign_incarnation() {
    let world = World::new(2);
    let mut state = UiFocusRuntimeState::new_session_restore_candidate();
    state
        .reconcile_mounted_participation(&world.snapshot)
        .unwrap();
    state
        .commit(
            state
                .plan(UiFocusRequest::Direct {
                    scope: world.scope,
                    participant: world.identities[1].0,
                    incarnation: world.identities[1].1,
                    cause: UiFocusCause::Direct,
                })
                .unwrap(),
        )
        .unwrap();
    let token = state.restoration_token().unwrap();
    let successor = world.with_reincarnated_participant(1);

    let receipt = state
        .reconcile_mounted_participation(&successor.snapshot)
        .unwrap();
    let fallback = receipt.transition().unwrap();
    assert_eq!(fallback.cause(), UiFocusCause::RebindFallback);
    assert_eq!(fallback.outcome(), UiFocusOutcome::Moved);
    assert_eq!(
        fallback.current().unwrap().participant(),
        successor.identities[0].0
    );

    let restore = state
        .commit(state.plan(UiFocusRequest::Restore(token)).unwrap())
        .unwrap();
    assert_eq!(
        restore.current().unwrap().participant(),
        successor.identities[0].0
    );
}

#[test]
fn rebind_preservation_emits_the_successor_mounted_receipt_for_host_placement() {
    let world = World::new(1);
    let mut state = UiFocusRuntimeState::new_session_restore_candidate();
    state
        .reconcile_mounted_participation(&world.snapshot)
        .unwrap();
    let initial = state
        .commit(state.plan(first_request(world.scope)).unwrap())
        .unwrap()
        .current()
        .unwrap();
    let successor = world.with_successor(None);
    let receipt = state
        .reconcile_mounted_participation(&successor.snapshot)
        .unwrap()
        .transition()
        .expect("a new mounted receipt requires exact host focus replacement");
    assert_eq!(receipt.cause(), UiFocusCause::RebindPreserved);
    assert_ne!(
        receipt.current().unwrap().mounted_target().node_receipt(),
        initial.mounted_target().node_receipt()
    );
}

#[test]
fn window_focus_and_pointer_modality_do_not_erase_semantic_focus() {
    let world = World::new(1);
    let mut state = UiFocusRuntimeState::new_session_restore_candidate();
    state
        .reconcile_mounted_participation(&world.snapshot)
        .unwrap();
    state.observe_window_focus(true);
    state
        .commit(
            state
                .plan(UiFocusRequest::Traverse {
                    scope: world.scope,
                    direction: UiFocusTraversalDirection::Forward,
                    wrap: true,
                })
                .unwrap(),
        )
        .unwrap();
    assert!(state.inspect().focus_visible());

    state.observe_window_focus(false);
    assert!(state.inspect().current().is_some());
    assert!(!state.inspect().focus_visible());
    state.observe_window_focus(true);
    assert!(state.inspect().focus_visible());
    state.observe_pointer_modality();
    assert!(state.inspect().current().is_some());
    assert!(!state.inspect().focus_visible());
}

struct World {
    snapshot: crate::mounting::UiMountedFocusParticipationSnapshot,
    scope: UiFocusScopeIdentity,
    identities: Vec<(
        UiFocusParticipantIdentity,
        worth_ui_host_contract::UiMountIncarnation,
    )>,
}

impl World {
    fn new(focusable_count: usize) -> Self {
        let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let mut identities = Vec::new();
        let mut participants = Vec::new();
        for index in 0..focusable_count {
            let mounted =
                worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
            let incarnation = worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap();
            identities.push((
                UiFocusParticipantIdentity::for_mounted_instance(mounted),
                incarnation,
            ));
            participants.push(crate::mounting::UiMountedFocusParticipant::new(
                crate::graph::UiGraphNodeIdentity::new(index as u64 + 1),
                surface,
                mounted,
                incarnation,
                issuer.receipt_for(mounted),
                crate::capability::ComponentFocusSupport::focusable(),
                crate::mounting::UiMountedFocusScope::ActiveSurface,
                index as u32,
            ));
        }
        Self {
            snapshot: crate::mounting::UiMountedFocusParticipationSnapshot::new(
                frame,
                participants,
                u32::try_from(focusable_count).unwrap(),
                Vec::new(),
            ),
            scope: UiFocusScopeIdentity::for_surface(surface),
            identities,
        }
    }

    fn with_reincarnated_participant(&self, target: usize) -> Self {
        self.with_successor(Some(target))
    }

    fn with_successor(&self, reincarnated: Option<usize>) -> Self {
        let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let participants = self
            .identities
            .iter()
            .enumerate()
            .map(|(index, (identity, incarnation))| {
                let incarnation = if Some(index) == reincarnated {
                    worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap()
                } else {
                    *incarnation
                };
                crate::mounting::UiMountedFocusParticipant::new(
                    crate::graph::UiGraphNodeIdentity::new(index as u64 + 1),
                    self.scope.semantic_surface(),
                    identity.mounted_instance(),
                    incarnation,
                    issuer.receipt_for(identity.mounted_instance()),
                    crate::capability::ComponentFocusSupport::focusable(),
                    crate::mounting::UiMountedFocusScope::ActiveSurface,
                    index as u32,
                )
            })
            .collect();
        Self {
            snapshot: crate::mounting::UiMountedFocusParticipationSnapshot::new(
                frame,
                participants,
                self.identities.len() as u32,
                Vec::new(),
            ),
            scope: self.scope,
            identities: self.identities.clone(),
        }
    }
}
