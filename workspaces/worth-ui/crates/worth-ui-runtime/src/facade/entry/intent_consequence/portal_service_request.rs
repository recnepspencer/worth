pub(super) fn portal_service_request(
    handoff: &crate::runtime::intent_execution::UiIntentConsequenceHandoff,
    destination: crate::capability::UiIntentRuntimeServiceDestination,
    presented_viewport: Option<crate::runtime::interaction::UiPresentedViewportGeometry>,
    resolved_owner: Option<(
        crate::graph::UiGraphNodeIdentity,
        worth_ui_host_contract::UiMountedInstanceIdentity,
    )>,
) -> crate::runtime::portal::UiPortalServiceRequest {
    let owner = resolved_owner.map_or_else(
        || {
            crate::runtime::portal::UiPortalOwnerIdentity::from_target(
                handoff.graph_node(),
                handoff.target(),
            )
        },
        |(graph_node, mounted_instance)| {
            crate::runtime::portal::UiPortalOwnerIdentity::from_mounted_owner(
                graph_node,
                mounted_instance,
            )
        },
    );
    let portal = crate::runtime::portal::UiPortalIdentity::for_owner(owner);
    let request = match destination {
        crate::capability::UiIntentRuntimeServiceDestination::OpenPortal => {
            crate::runtime::portal::UiPortalServiceRequest::open(
                portal,
                handoff.idempotency(),
                handoff.target().geometry(),
                presented_viewport,
                handoff.target().surface(),
            )
        }
        crate::capability::UiIntentRuntimeServiceDestination::ClosePortal => {
            crate::runtime::portal::UiPortalServiceRequest::close(
                portal,
                handoff.idempotency(),
                crate::runtime::portal::UiPortalDismissalCause::ExplicitOwnerRequest,
                handoff.target().surface(),
            )
        }
        crate::capability::UiIntentRuntimeServiceDestination::InvokeCommand => {
            unreachable!("command consequences never construct mounted portal service requests")
        }
    };
    match destination {
        crate::capability::UiIntentRuntimeServiceDestination::OpenPortal => {
            request.with_declared_portal(handoff.authored_portal_declaration())
        }
        crate::capability::UiIntentRuntimeServiceDestination::ClosePortal
        | crate::capability::UiIntentRuntimeServiceDestination::InvokeCommand => request,
    }
}

pub(super) fn portal_placement_stop_reason(
    denial: crate::runtime::portal::UiPortalPlacementDenial,
) -> crate::runtime::intent_execution::UiIntentPortalPlacementStopReason {
    use crate::runtime::intent_execution::UiIntentPortalPlacementStopReason as Stop;
    match denial {
        crate::runtime::portal::UiPortalPlacementDenial::MissingPresentedAnchor => {
            Stop::MissingPresentedAnchor
        }
        crate::runtime::portal::UiPortalPlacementDenial::MissingPresentedViewport => {
            Stop::MissingPresentedViewport
        }
        crate::runtime::portal::UiPortalPlacementDenial::IncompatibleCoordinateSpace => {
            Stop::IncompatibleCoordinateSpace
        }
        crate::runtime::portal::UiPortalPlacementDenial::EmptyAnchor => Stop::EmptyAnchor,
        crate::runtime::portal::UiPortalPlacementDenial::InsufficientViewport => {
            Stop::InsufficientViewport
        }
        crate::runtime::portal::UiPortalPlacementDenial::UnknownParent => Stop::UnknownPortalParent,
        crate::runtime::portal::UiPortalPlacementDenial::LayerDepthExhausted => {
            Stop::PortalLayerDepthExhausted
        }
    }
}
