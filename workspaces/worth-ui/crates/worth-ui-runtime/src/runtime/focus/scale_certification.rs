pub(crate) fn focus_scale_evidence() -> (u64, u64, bool) {
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let mut identities = Vec::new();
    let mut participants = Vec::new();
    for index in 0..128_u64 {
        let mounted = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let incarnation = worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap();
        identities.push((
            super::UiFocusParticipantIdentity::for_mounted_instance(mounted),
            incarnation,
        ));
        participants.push(crate::mounting::UiMountedFocusParticipant::new(
            crate::graph::UiGraphNodeIdentity::new(index + 1),
            surface,
            mounted,
            incarnation,
            issuer.receipt_for(mounted),
            crate::capability::ComponentFocusSupport::focusable(),
            crate::mounting::UiMountedFocusScope::ActiveSurface,
            index as u32,
        ));
    }
    let snapshot = crate::mounting::UiMountedFocusParticipationSnapshot::new(
        frame,
        participants,
        128,
        Vec::new(),
    );
    let mut state = super::UiFocusRuntimeState::new_session_restore_candidate();
    let installed = state
        .reconcile_mounted_participation(&snapshot)
        .expect("focus scale snapshot reconciles");
    let scope = super::UiFocusScopeIdentity::for_surface(surface);
    let first = state
        .plan(super::UiFocusRequest::Direct {
            scope,
            participant: identities[0].0,
            incarnation: identities[0].1,
            cause: super::UiFocusCause::Direct,
        })
        .expect("first participant routes");
    state.commit(first).expect("first focus commits");
    let traversal = state
        .plan(super::UiFocusRequest::Traverse {
            scope,
            direction: super::UiFocusTraversalDirection::Forward,
            wrap: true,
        })
        .expect("indexed traversal plans");
    let receipt = state.commit(traversal).expect("traversal commits");
    let released = state.shutdown();
    let terminal_zero = state.participants.is_empty()
        && state.participant_index.is_empty()
        && state.current.is_none()
        && state.active_descendant.is_none()
        && state.pending_portal.is_empty()
        && state.portal_restorations.is_empty();
    (
        installed.participants_installed() as u64,
        receipt.participants_visited() as u64,
        released == 129 && terminal_zero,
    )
}
