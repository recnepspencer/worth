pub(crate) fn portal_scale_evidence() -> (u64, u64, bool) {
    let mut state = super::UiPortalRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
    );
    let geometry = presented_geometry();
    let surface = semantic_surface();
    let portals = (0..4_u64)
        .map(|index| {
            super::UiPortalIdentity::for_owner(super::UiPortalOwnerIdentity::for_test(
                index + 1,
                index + 101,
            ))
        })
        .collect::<Vec<_>>();

    for (index, portal) in portals.iter().copied().enumerate() {
        let request = if index == 0 {
            super::UiPortalServiceRequest::open(
                portal,
                idempotency(index as u64 + 1),
                geometry,
                Some(viewport_bounds(geometry)),
                surface,
            )
        } else {
            super::UiPortalServiceRequest::open_nested(
                portal,
                idempotency(index as u64 + 1),
                geometry,
                viewport_bounds(geometry),
                surface,
                portals[index - 1],
                super::UiPortalInputShielding::ContentBounds,
            )
        };
        let transition = state.prepare(request).expect("four portal layers fit");
        state
            .commit_published(transition)
            .expect("scale portal transition remains current");
    }
    let active_layers = state.active_count() as u64;
    let close = state
        .prepare(super::UiPortalServiceRequest::close(
            portals[0],
            idempotency(10),
            super::UiPortalDismissalCause::ExplicitOwnerRequest,
            surface,
        ))
        .expect("closing the root resolves its exact descendants");
    let affected_layers = close.closed_descendants().len() as u64 + 1;
    state
        .commit_published(close)
        .expect("the four-layer close remains current");
    let shutdown = state.shutdown();
    (
        active_layers,
        affected_layers,
        shutdown.final_active_records() == 0,
    )
}

fn idempotency(
    lineage: u64,
) -> crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity {
    crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, lineage)
}

fn semantic_surface() -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
    worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("semantic surface")
}

fn presented_geometry() -> crate::runtime::interaction::UiPresentedInteractionGeometry {
    let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound()
        .expect("binding generation");
    let presentation = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().expect("host surface"),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().expect("frame"),
        binding,
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    );
    crate::runtime::interaction::UiPresentedInteractionGeometry::for_test(presentation)
}

fn viewport_bounds(
    geometry: crate::runtime::interaction::UiPresentedInteractionGeometry,
) -> crate::runtime::interaction::UiPresentedViewportGeometry {
    crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
        geometry.clip_bounds().adopted().canonical_box(),
        geometry.presentation(),
    )
}
